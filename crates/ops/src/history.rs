use std::collections::{BTreeMap, BTreeSet, VecDeque};

use scorecraft_doc::{Library, ScoreId};

use crate::{Applied, OpError, Touched, Transaction};

/// Undo steps kept per scope by default.
pub const DEFAULT_LIMIT: usize = 500;

/// Which undo stack a step belongs to: a score's (its markings and pages), or the library's
/// (folders, lists, imports, anything that spans records). Undo in the reader undoes the open
/// score's last step; undo in the library undoes the library's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    Library,
    Score(ScoreId),
}

impl Scope {
    /// The scope of a change to these records: one score's, or the library's.
    pub fn of(touched: &BTreeSet<Touched>) -> Scope {
        let mut it = touched.iter();
        match (it.next(), it.next()) {
            (Some(Touched::Score(id)), None) => Scope::Score(*id),
            _ => Scope::Library,
        }
    }
}

#[derive(Clone, Debug)]
struct Step {
    /// Global order, so "undo the last thing" works across scopes.
    seq: u64,
    label: String,
    tx: Transaction,
}

#[derive(Clone, Debug, Default)]
struct Stack {
    undo: VecDeque<Step>,
    redo: Vec<Step>,
}

/// What an undo or redo did.
#[derive(Clone, Debug, PartialEq)]
pub struct Undone {
    pub label: String,
    pub scope: Scope,
    pub touched: BTreeSet<Touched>,
}

/// Undo and redo stacks per [`Scope`], bounded. Lives for the session, so a score's history
/// survives closing and reopening it (architecture §6). Remote (sync) changes are applied with
/// [`Transaction::apply`] directly and never enter a stack.
#[derive(Clone, Debug)]
pub struct History {
    stacks: BTreeMap<Scope, Stack>,
    limit: usize,
    next_seq: u64,
}

impl Default for History {
    fn default() -> Self {
        History::new(DEFAULT_LIMIT)
    }
}

impl History {
    /// A history keeping at most `limit` undo steps per scope (at least one).
    pub fn new(limit: usize) -> History {
        History {
            stacks: BTreeMap::new(),
            limit: limit.max(1),
            next_seq: 0,
        }
    }

    /// Applies `tx` as one undoable step labelled `label` ("Rotate page"). An empty transaction
    /// is applied but records nothing. A new step clears that scope's redo stack.
    pub fn commit(
        &mut self,
        lib: &mut Library,
        label: &str,
        tx: Transaction,
    ) -> Result<Applied, OpError> {
        let applied = tx.apply(lib)?;
        if applied.inverse.is_empty() {
            return Ok(applied);
        }
        let scope = Scope::of(&applied.touched);
        let seq = self.bump();
        let limit = self.limit;
        let stack = self.stacks.entry(scope).or_default();
        stack.redo.clear();
        stack.undo.push_back(Step {
            seq,
            label: label.to_owned(),
            tx: applied.inverse.clone(),
        });
        while stack.undo.len() > limit {
            stack.undo.pop_front();
        }
        Ok(applied)
    }

    fn bump(&mut self) -> u64 {
        self.next_seq = self.next_seq.saturating_add(1);
        self.next_seq
    }

    /// The scope whose newest undo step is the most recent, among `scope` (or all scopes).
    fn pick(&self, scope: Option<Scope>, redo: bool) -> Option<Scope> {
        let top = |s: &Stack| {
            if redo {
                s.redo.last().map(|st| st.seq)
            } else {
                s.undo.back().map(|st| st.seq)
            }
        };
        match scope {
            Some(sc) => self.stacks.get(&sc).and_then(top).map(|_| sc),
            None => self
                .stacks
                .iter()
                .filter_map(|(sc, st)| top(st).map(|seq| (seq, *sc)))
                .max()
                .map(|(_, sc)| sc),
        }
    }

    /// Undoes the newest step in `scope`, or the newest step anywhere for `None`. A step that no
    /// longer applies (a sync change removed what it touched) is dropped with the error.
    pub fn undo(&mut self, lib: &mut Library, scope: Option<Scope>) -> Result<Undone, OpError> {
        let sc = self.pick(scope, false).ok_or(OpError::NothingToUndo)?;
        let seq = self.bump();
        let stack = self.stacks.entry(sc).or_default();
        let step = stack.undo.pop_back().ok_or(OpError::NothingToUndo)?;
        let applied = step.tx.apply(lib)?;
        stack.redo.push(Step {
            seq,
            label: step.label.clone(),
            tx: applied.inverse,
        });
        Ok(Undone {
            label: step.label,
            scope: sc,
            touched: applied.touched,
        })
    }

    /// Redoes the newest undone step in `scope`, or anywhere for `None`.
    pub fn redo(&mut self, lib: &mut Library, scope: Option<Scope>) -> Result<Undone, OpError> {
        let sc = self.pick(scope, true).ok_or(OpError::NothingToRedo)?;
        let seq = self.bump();
        let stack = self.stacks.entry(sc).or_default();
        let step = stack.redo.pop().ok_or(OpError::NothingToRedo)?;
        let applied = step.tx.apply(lib)?;
        stack.undo.push_back(Step {
            seq,
            label: step.label.clone(),
            tx: applied.inverse,
        });
        Ok(Undone {
            label: step.label,
            scope: sc,
            touched: applied.touched,
        })
    }

    /// The label of the step [`History::undo`] would undo.
    pub fn undo_label(&self, scope: Option<Scope>) -> Option<&str> {
        let sc = self.pick(scope, false)?;
        self.stacks
            .get(&sc)
            .and_then(|s| s.undo.back())
            .map(|s| s.label.as_str())
    }

    pub fn redo_label(&self, scope: Option<Scope>) -> Option<&str> {
        let sc = self.pick(scope, true)?;
        self.stacks
            .get(&sc)
            .and_then(|s| s.redo.last())
            .map(|s| s.label.as_str())
    }

    pub fn can_undo(&self, scope: Option<Scope>) -> bool {
        self.pick(scope, false).is_some()
    }

    pub fn can_redo(&self, scope: Option<Scope>) -> bool {
        self.pick(scope, true).is_some()
    }

    /// Undo steps in `scope`.
    pub fn depth(&self, scope: Scope) -> usize {
        self.stacks.get(&scope).map_or(0, |s| s.undo.len())
    }

    /// Forgets a scope's steps (a score emptied from Recently Deleted).
    pub fn forget(&mut self, scope: Scope) {
        self.stacks.remove(&scope);
    }

    pub fn clear(&mut self) {
        self.stacks.clear();
    }
}
