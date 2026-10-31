//! The control channel (`docs/control-protocol.md`): JSON lines over an authenticated loopback
//! TCP socket. Connections run on their own threads; each request is handed to the app's UI
//! thread through a [`Request`] and answered when the app replies, so the app stays the single
//! owner of its session.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use serde_json::{Value, json};

/// Longest request line accepted.
pub const MAX_LINE: usize = 8 * 1024 * 1024;
/// Most connections open at once.
pub const MAX_CONNECTIONS: usize = 8;
/// Longest the app may take to answer one request.
pub const REPLY_TIMEOUT: Duration = Duration::from_secs(60);

/// An answer: a result, or an error code and message.
pub type Reply = Result<Value, (String, String)>;

/// How long a new connection has to send its token.
pub const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

/// One request for the app: a method and params, and where to send the answer.
pub struct Request {
    pub method: String,
    pub params: Value,
    reply: Sender<Reply>,
}

impl Request {
    /// Answers with a result.
    pub fn ok(self, result: Value) {
        let _ = self.reply.send(Ok(result));
    }

    /// Answers with an error code and message.
    pub fn err(self, code: &str, message: impl Into<String>) {
        let _ = self.reply.send(Err((code.to_owned(), message.into())));
    }
}

impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}

/// A running control server.
pub struct Server {
    pub addr: SocketAddr,
    pub token: String,
    /// Requests waiting for the app; drain it every frame.
    pub requests: Receiver<Request>,
}

/// A 256-bit token as 64 hex characters from the OS random source, unless
/// `SCORECRAFT_CONTROL_TOKEN` is set. `None` if the OS has no randomness to give.
pub fn new_token() -> Option<String> {
    if let Ok(t) = std::env::var("SCORECRAFT_CONTROL_TOKEN")
        && !t.is_empty()
    {
        return Some(t);
    }
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).ok()?;
    Some(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Starts listening on 127.0.0.1 (`port` 0 picks a free one). `wake` is called whenever a
/// request arrives, so the app can schedule a frame.
pub fn start(
    port: u16,
    token: String,
    wake: Arc<dyn Fn() + Send + Sync>,
) -> std::io::Result<Server> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let addr = listener.local_addr()?;
    let (tx, rx) = mpsc::channel();
    let open = Arc::new(AtomicUsize::new(0));
    let accept_token = token.clone();
    std::thread::Builder::new()
        .name("scorecraft-control".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                if open.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
                    let mut s = stream;
                    let _ = writeln!(
                        s,
                        "{}",
                        error_line(Value::Null, "busy", "too many connections")
                    );
                    continue;
                }
                open.fetch_add(1, Ordering::SeqCst);
                let (tx, token, wake, count) =
                    (tx.clone(), accept_token.clone(), wake.clone(), open.clone());
                let spawned = std::thread::Builder::new()
                    .name("scorecraft-control-conn".into())
                    .spawn(move || {
                        let _ = serve(stream, &token, &tx, &*wake);
                        count.fetch_sub(1, Ordering::SeqCst);
                    });
                if spawned.is_err() {
                    // The closure (and its counter decrement) was dropped unrun.
                    open.fetch_sub(1, Ordering::SeqCst);
                }
            }
        })?;
    Ok(Server {
        addr,
        token,
        requests: rx,
    })
}

fn error_line(id: Value, code: &str, message: &str) -> String {
    json!({ "id": id, "error": { "code": code, "message": message } }).to_string()
}

/// Reads one line of at most [`MAX_LINE`] bytes; `None` at end of stream.
fn read_line(r: &mut impl BufRead) -> std::io::Result<Option<Result<String, ()>>> {
    let mut buf = Vec::new();
    let n = r
        .by_ref()
        .take(MAX_LINE as u64 + 1)
        .read_until(b'\n', &mut buf)?;
    if n == 0 {
        return Ok(None);
    }
    if buf.len() > MAX_LINE {
        // Skip the rest of the long line so the next read starts at a request.
        loop {
            let mut sink = Vec::new();
            let n = r.by_ref().take(64 * 1024).read_until(b'\n', &mut sink)?;
            if n == 0 || sink.last() == Some(&b'\n') {
                break;
            }
        }
        return Ok(Some(Err(())));
    }
    Ok(Some(String::from_utf8(buf).map_err(|_| ())))
}

/// Constant-time comparison, so the token can't be guessed byte by byte from timing.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}

fn serve(
    stream: TcpStream,
    token: &str,
    tx: &Sender<Request>,
    wake: &dyn Fn(),
) -> std::io::Result<()> {
    // A connection that never authenticates must not hold a slot for long.
    stream.set_read_timeout(Some(AUTH_TIMEOUT))?;
    let mut out = stream.try_clone()?;
    let mut lines = BufReader::new(stream);
    // Authenticate first.
    let authed = match read_line(&mut lines)? {
        Some(Ok(l)) => serde_json::from_str::<Value>(&l)
            .ok()
            .and_then(|v| {
                v.get("auth")
                    .and_then(Value::as_str)
                    .map(|t| same(t, token))
            })
            .unwrap_or(false),
        _ => false,
    };
    if !authed {
        writeln!(
            out,
            "{}",
            error_line(
                Value::Null,
                "unauthorized",
                "send {\"auth\":\"<token>\"} first"
            )
        )?;
        return Ok(());
    }
    lines.get_ref().set_read_timeout(None)?;
    writeln!(out, "{}", json!({ "ok": true }))?;
    while let Some(line) = read_line(&mut lines)? {
        let Ok(line) = line else {
            writeln!(
                out,
                "{}",
                error_line(Value::Null, "bad_request", "line too long or not UTF-8")
            )?;
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Err(e) => error_line(Value::Null, "bad_request", &format!("not JSON: {e}")),
            Ok(req) => {
                let id = req.get("id").cloned().unwrap_or(Value::Null);
                match req.get("method").and_then(Value::as_str) {
                    None => error_line(id, "bad_request", "`method` is required"),
                    Some(method) => {
                        let (rtx, rrx) = mpsc::channel();
                        let request = Request {
                            method: method.chars().take(128).collect(),
                            params: req.get("params").cloned().unwrap_or(Value::Null),
                            reply: rtx,
                        };
                        if tx.send(request).is_err() {
                            return Ok(());
                        }
                        wake();
                        match rrx.recv_timeout(REPLY_TIMEOUT) {
                            Ok(Ok(result)) => json!({ "id": id, "result": result }).to_string(),
                            Ok(Err((code, msg))) => error_line(id, &code, &msg),
                            Err(_) => error_line(id, "timeout", "the app did not answer"),
                        }
                    }
                }
            }
        };
        writeln!(out, "{reply}")?;
    }
    Ok(())
}

/// The shared part of answering requests: `command`, `command_batch` and `command_list` over
/// any command runner. The app handles the `ui.*` methods that need its window itself.
pub fn handle_common(
    method: &str,
    params: &Value,
    run: &mut dyn FnMut(&str, &Value) -> Reply,
    list: &dyn Fn() -> Value,
) -> Option<Reply> {
    match method {
        "command" => {
            let Some(id) = params.get("id").and_then(Value::as_str) else {
                return Some(Err(("bad_request".into(), "`id` is required".into())));
            };
            let p = params.get("params").cloned().unwrap_or(Value::Null);
            Some(run(id, &p))
        }
        "command_batch" => {
            let Some(steps) = params.get("steps").and_then(Value::as_array) else {
                return Some(Err(("bad_request".into(), "`steps` must be a list".into())));
            };
            if steps.len() > 1000 {
                return Some(Err(("bad_request".into(), "at most 1000 steps".into())));
            }
            let stop = params
                .get("stop_on_error")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let mut results = Vec::new();
            for step in steps {
                let id = step.get("id").and_then(Value::as_str).unwrap_or("");
                let p = step.get("params").cloned().unwrap_or(Value::Null);
                match run(id, &p) {
                    Ok(v) => results.push(json!({ "result": v })),
                    Err((code, message)) => {
                        results.push(json!({ "error": { "code": code, "message": message } }));
                        if stop {
                            break;
                        }
                    }
                }
            }
            Some(Ok(json!({ "results": results })))
        }
        "command_list" => {
            let filter = params.get("filter").and_then(Value::as_str).unwrap_or("");
            let all = list();
            let items: Vec<Value> = all
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter(|c| {
                            c.get("id")
                                .and_then(Value::as_str)
                                .is_some_and(|id| id.starts_with(filter))
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            Some(Ok(Value::Array(items)))
        }
        _ => None,
    }
}

/// A blocking client, for tests and scripts.
pub struct Client {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    next: u64,
}

impl Client {
    pub fn connect(addr: SocketAddr, token: &str) -> std::io::Result<Client> {
        let s = TcpStream::connect(addr)?;
        s.set_read_timeout(Some(REPLY_TIMEOUT))?;
        let mut c = Client {
            reader: BufReader::new(s.try_clone()?),
            writer: s,
            next: 1,
        };
        let r = c.line(&json!({ "auth": token }))?;
        if r.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                r.to_string(),
            ));
        }
        Ok(c)
    }

    fn line(&mut self, v: &Value) -> std::io::Result<Value> {
        writeln!(self.writer, "{v}")?;
        let mut l = String::new();
        self.reader.read_line(&mut l)?;
        serde_json::from_str(&l)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Sends a request and returns the whole reply (`result` or `error`).
    pub fn call(&mut self, method: &str, params: Value) -> std::io::Result<Value> {
        let id = self.next;
        self.next += 1;
        self.line(&json!({ "id": id, "method": method, "params": params }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake app: answers on its own thread.
    fn serve_fake(server: Server) {
        std::thread::spawn(move || {
            while let Ok(req) = server.requests.recv() {
                let mut run = |id: &str, p: &Value| -> Result<Value, (String, String)> {
                    if id == "fail" {
                        Err(("bad_params".into(), "no".into()))
                    } else {
                        Ok(json!({ "ran": id, "params": p }))
                    }
                };
                let list = || json!([{ "id": "view.turn.next" }, { "id": "score.open" }]);
                match handle_common(&req.method, &req.params, &mut run, &list) {
                    Some(Ok(v)) => req.ok(v),
                    Some(Err((c, m))) => req.err(&c, m),
                    None => req.err("unknown_method", "no such method"),
                }
            }
        });
    }

    fn server() -> Server {
        start(0, "secret".into(), Arc::new(|| {})).unwrap()
    }

    #[test]
    fn auth_is_required() {
        let s = server();
        assert!(Client::connect(s.addr, "wrong").is_err());
        let (addr, token) = (s.addr, s.token.clone());
        serve_fake(s);
        assert!(Client::connect(addr, &token).is_ok());
    }

    #[test]
    fn commands_batches_and_lists() {
        let s = server();
        let addr = s.addr;
        serve_fake(s);
        let mut c = Client::connect(addr, "secret").unwrap();
        let r = c
            .call("command", json!({ "id": "view.turn.next" }))
            .unwrap();
        assert_eq!(r["result"]["ran"], "view.turn.next");
        let r = c.call("command", json!({ "id": "fail" })).unwrap();
        assert_eq!(r["error"]["code"], "bad_params");
        let r = c
            .call(
                "command_batch",
                json!({ "steps": [{ "id": "a" }, { "id": "fail" }, { "id": "b" }] }),
            )
            .unwrap();
        assert_eq!(r["result"]["results"].as_array().unwrap().len(), 2);
        let r = c
            .call("command_list", json!({ "filter": "view." }))
            .unwrap();
        assert_eq!(r["result"].as_array().unwrap().len(), 1);
        let r = c.call("nope", json!({})).unwrap();
        assert_eq!(r["error"]["code"], "unknown_method");
    }

    #[test]
    fn garbage_lines_get_errors_not_hangups() {
        let s = server();
        let addr = s.addr;
        serve_fake(s);
        let mut c = Client::connect(addr, "secret").unwrap();
        writeln!(c.writer, "not json").unwrap();
        let mut l = String::new();
        c.reader.read_line(&mut l).unwrap();
        assert!(l.contains("bad_request"));
        let r = c.call("command", json!({})).unwrap();
        assert_eq!(r["error"]["code"], "bad_request");
    }

    #[test]
    fn tokens_are_long_and_differ() {
        let (a, b) = (new_token().unwrap(), new_token().unwrap());
        if std::env::var("SCORECRAFT_CONTROL_TOKEN").is_err() {
            assert_eq!(a.len(), 64);
            assert_ne!(a, b);
        }
        assert!(same("abc", "abc") && !same("abc", "abd") && !same("abc", "ab"));
    }
}
