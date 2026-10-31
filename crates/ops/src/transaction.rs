use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use scorecraft_doc::Library;

use crate::{Op, OpError, Touched};

/// Ops applied together as one step: all or nothing, one undo step per command (architecture
/// §6).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub ops: Vec<Op>,
}

/// A transaction that has been applied: what undoes it and which records it touched.
#[derive(Clone, Debug, PartialEq)]
pub struct Applied {
    /// The inverse ops, already in undo order.
    pub inverse: Transaction,
    pub touched: BTreeSet<Touched>,
}

impl Transaction {
    pub fn new(ops: Vec<Op>) -> Transaction {
        Transaction { ops }
    }

    pub fn single(op: Op) -> Transaction {
        Transaction { ops: vec![op] }
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// The records the ops change.
    pub fn touches(&self) -> BTreeSet<Touched> {
        self.ops.iter().map(Op::touches).collect()
    }

    /// Applies every op in order. If one fails, the ones already applied are undone and the
    /// library is exactly as it was.
    pub fn apply(self, lib: &mut Library) -> Result<Applied, OpError> {
        let touched = self.touches();
        let mut inverse = Vec::with_capacity(self.ops.len());
        for op in self.ops {
            match op.apply(lib) {
                Ok(inv) => inverse.push(inv),
                Err(e) => {
                    roll_back(lib, inverse);
                    return Err(e);
                }
            }
        }
        inverse.reverse();
        Ok(Applied {
            inverse: Transaction { ops: inverse },
            touched,
        })
    }
}

/// Undoes already-applied ops (given in apply order). Each inverse applies to the state its op
/// produced, so it cannot fail; if it somehow does, the remaining ones still run.
fn roll_back(lib: &mut Library, applied_inverses: Vec<Op>) {
    for inv in applied_inverses.into_iter().rev() {
        let _ = inv.apply(lib);
    }
}
