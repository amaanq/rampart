//! LMTP state-machine tests. DB-free via a MockHandler that returns
//! preset verdicts. Drives `handle_session_io` through an in-memory
//! `tokio::io::duplex` pair.
//!
//! `tokio::io::duplex` returns `(client, server)`. The test speaks LMTP
//! on `client`; `handle_session_io` reads/writes on `server`.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

use rampart::config::Config;
use rampart::mailer::MemoryMailer;
use rampart::worker::{
    DeliveryHandler, WorkerState,
    lmtp::{
        INTERNAL_DOMAIN, MAX_LINE_LEN, MAX_MESSAGE_SIZE, handle_session_io, serve_with_listener,
    },
    pipeline::{Delivery, Verdict},
    resubmit::MemorySubmit,
};

/// Returns a fully-constructed WorkerState whose pool is never resolved
/// (we never call `.get()`). The DeliveryHandler is the only place
/// pipeline-style queries would happen, and the MockHandler avoids them.
fn make_state() -> WorkerState {
    let pool =
        rampart::db::build_pool("host=/tmp/rampart-no-such-socket-shouldnt-connect dbname=fake")
            .expect("pool build (no connection attempt)");
    let cfg = Config {
        database_url: "host=/tmp/rampart-no-such-socket dbname=fake".into(),
        listen: "127.0.0.1:0".parse().unwrap(),
        public_origin: "http://localhost".into(),
        static_dir: "static".into(),
        sieve_output_path: None,
        smtp_host: "localhost".into(),
        smtp_port: 465,
        smtp_user: "x@x".into(),
        smtp_password_file: None,
        notifier_from: "\"rampart\" <x@x>".into(),
        webauthn_rp_id: "localhost".into(),
        lmtp_listen: "127.0.0.1:0".parse().unwrap(),
        stalwart_hostname: "test.example".into(),
        lmtp_drain_secs: 20,
        stalwart_jmap_base_url: None,
        stalwart_admin_username: "admin".into(),
        stalwart_admin_password_file: None,
        verp_key: b"test-key-32-bytes-long-padding-padding".to_vec(),
    };
    WorkerState {
        pool,
        config: Arc::new(cfg),
        mailer: Arc::new(MemoryMailer::new()),
        submit: Arc::new(MemorySubmit::new()),
    }
}

#[derive(Default)]
struct MockHandler {
    preset: Mutex<VecDeque<Verdict>>,
    calls: Mutex<Vec<Delivery>>,
}

impl MockHandler {
    fn with_verdicts(verdicts: impl IntoIterator<Item = Verdict>) -> Self {
        Self {
            preset: Mutex::new(verdicts.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        }
    }
    fn calls(&self) -> std::sync::MutexGuard<'_, Vec<Delivery>> {
        self.calls.lock().unwrap()
    }
}

#[async_trait]
impl DeliveryHandler for MockHandler {
    async fn handle(&self, _state: &WorkerState, d: Delivery) -> Verdict {
        self.calls.lock().unwrap().push(Delivery {
            rcpt: d.rcpt,
            mail_from: d.mail_from.clone(),
            raw: d.raw.clone(),
        });
        self.preset
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Verdict::Delivered)
    }
}

/// Spawn `handle_session_io` on the server-side of a duplex pair and
/// return the client side for the test to drive. Returns the join handle
/// so the test can await session completion (e.g. on QUIT).
fn spawn_session(
    state: WorkerState,
    handler: Arc<MockHandler>,
) -> (
    tokio::io::DuplexStream,
    tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let (client, server) = tokio::io::duplex(64 * 1024);
    let (server_r, server_w) = tokio::io::split(server);
    let join = tokio::spawn(async move {
        handle_session_io(state, BufReader::new(server_r), server_w, handler.as_ref()).await
    });
    (client, join)
}

/// Read until we've collected at least `lines` newline-terminated chunks
/// or the stream closes. Returns the accumulated text.
async fn read_n_lines<R: tokio::io::AsyncRead + Unpin>(r: &mut R, lines: usize) -> String {
    let mut buf = String::new();
    let mut tmp = [0u8; 4096];
    while buf.matches('\n').count() < lines {
        let n = r.read(&mut tmp).await.unwrap_or(0);
        if n == 0 {
            break;
        }
        buf.push_str(&String::from_utf8_lossy(&tmp[..n]));
    }
    buf
}

#[tokio::test]
async fn session_accepts_known_rcpt() {
    let state = make_state();
    let mock = Arc::new(MockHandler::with_verdicts([Verdict::Delivered]));
    let (mut client, join) = spawn_session(state, mock.clone());

    // greeting (220)
    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"LHLO test.example\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 3).await; // 250-...250 8BITMIME
    client
        .write_all(b"MAIL FROM:<ext@sender.test>\r\n")
        .await
        .unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client
        .write_all(format!("RCPT TO:<rampart-42@{INTERNAL_DOMAIN}>\r\n").as_bytes())
        .await
        .unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("250"), "rcpt accept: {resp}");
    client.write_all(b"DATA\r\n").await.unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("354"), "data: {resp}");
    client
        .write_all(b"From: <ext@sender.test>\r\nSubject: hi\r\n\r\nbody\r\n.\r\n")
        .await
        .unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("250"), "post-DATA per-rcpt: {resp}");
    client.write_all(b"QUIT\r\n").await.unwrap();
    drop(client);
    let _ = join.await;

    let calls = mock.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].mail_from, "ext@sender.test");
}

#[tokio::test]
async fn session_rejects_unknown_rcpt() {
    let state = make_state();
    let mock = Arc::new(MockHandler::with_verdicts([]));
    let (mut client, join) = spawn_session(state, mock.clone());

    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"LHLO test.example\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 3).await;
    client.write_all(b"MAIL FROM:<x@y>\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client
        .write_all(format!("RCPT TO:<foo@{INTERNAL_DOMAIN}>\r\n").as_bytes())
        .await
        .unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("550"), "unknown rcpt: {resp}");
    client.write_all(b"QUIT\r\n").await.unwrap();
    drop(client);
    let _ = join.await;

    assert!(
        mock.calls().is_empty(),
        "handler must not be called on rejected rcpt"
    );
}

#[tokio::test]
async fn rset_between_transactions() {
    let state = make_state();
    let mock = Arc::new(MockHandler::with_verdicts([Verdict::Delivered]));
    let (mut client, join) = spawn_session(state, mock.clone());

    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"LHLO t\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 3).await;
    client.write_all(b"MAIL FROM:<a@a>\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"RSET\r\n").await.unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("250"), "rset: {resp}");

    // Trying DATA after RSET without MAIL/RCPT must fail.
    client.write_all(b"DATA\r\n").await.unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("503"), "data after rset must 503: {resp}");

    // New transaction works.
    client.write_all(b"MAIL FROM:<b@b>\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client
        .write_all(format!("RCPT TO:<rampart-1@{INTERNAL_DOMAIN}>\r\n").as_bytes())
        .await
        .unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"DATA\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client
        .write_all(b"From: x\r\n\r\nb\r\n.\r\n")
        .await
        .unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("250"), "second txn: {resp}");
    client.write_all(b"QUIT\r\n").await.unwrap();
    drop(client);
    let _ = join.await;

    assert_eq!(
        mock.calls().len(),
        1,
        "only second txn should reach handler"
    );
}

#[tokio::test]
async fn max_message_size_returns_552() {
    // Force a body that exceeds MAX_MESSAGE_SIZE.
    let state = make_state();
    let mock = Arc::new(MockHandler::with_verdicts([]));
    let (mut client, join) = spawn_session(state, mock.clone());

    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"LHLO t\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 3).await;
    client.write_all(b"MAIL FROM:<a@a>\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client
        .write_all(format!("RCPT TO:<rampart-1@{INTERNAL_DOMAIN}>\r\n").as_bytes())
        .await
        .unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"DATA\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;

    // Send headers, then chunked body until past the cap. Use lines just
    // under MAX_LINE_LEN so the line-len guard doesn't trip first.
    client.write_all(b"Subject: big\r\n\r\n").await.unwrap();
    let chunk = vec![b'x'; MAX_LINE_LEN - 4];
    let mut total: usize = 0;
    while total <= MAX_MESSAGE_SIZE {
        client.write_all(&chunk).await.unwrap();
        client.write_all(b"\r\n").await.unwrap();
        total += chunk.len() + 2;
    }
    client.write_all(b".\r\n").await.unwrap();
    let resp = read_n_lines(&mut client, 1).await;
    assert!(resp.starts_with("552"), "oversize body: {resp}");

    client.write_all(b"QUIT\r\n").await.unwrap();
    drop(client);
    let _ = join.await;

    assert!(
        mock.calls().is_empty(),
        "oversize body must not reach handler"
    );
}

#[tokio::test]
async fn line_too_long_closes_session() {
    let state = make_state();
    let mock = Arc::new(MockHandler::with_verdicts([]));
    let (mut client, join) = spawn_session(state, mock.clone());

    let _ = read_n_lines(&mut client, 1).await;
    let oversized = vec![b'x'; MAX_LINE_LEN + 16];
    // Send a long line with no terminator — handler errors out and closes
    // the stream once it has read MAX_LINE_LEN bytes, so the tail of the
    // write may BrokenPipe. That's the expected outcome.
    let _ = client.write_all(&oversized).await;
    drop(client);
    // handle_session_io returns Err on InvalidData; that's a normal end here.
    let _ = join.await;
    assert!(mock.calls().is_empty());
}

#[tokio::test]
async fn multi_rcpt_per_transaction() {
    let state = make_state();
    let mock = Arc::new(MockHandler::with_verdicts([
        Verdict::Delivered,
        Verdict::Perm {
            internal: "nope".into(),
            smtp: "nope",
        },
        Verdict::Delivered,
    ]));
    let (mut client, join) = spawn_session(state, mock.clone());

    let _ = read_n_lines(&mut client, 1).await;
    client.write_all(b"LHLO t\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 3).await;
    client.write_all(b"MAIL FROM:<x@x>\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    for id in [1, 2, 3] {
        client
            .write_all(format!("RCPT TO:<rampart-{id}@{INTERNAL_DOMAIN}>\r\n").as_bytes())
            .await
            .unwrap();
        let resp = read_n_lines(&mut client, 1).await;
        assert!(resp.starts_with("250"), "rcpt {id}: {resp}");
    }
    client.write_all(b"DATA\r\n").await.unwrap();
    let _ = read_n_lines(&mut client, 1).await;
    client
        .write_all(b"From: x\r\n\r\nb\r\n.\r\n")
        .await
        .unwrap();
    // Three per-RCPT responses (250 / 550 / 250).
    let resp = read_n_lines(&mut client, 3).await;
    let lines: Vec<&str> = resp.lines().collect();
    assert!(lines[0].starts_with("250"), "{}", lines[0]);
    assert!(lines[1].starts_with("550"), "{}", lines[1]);
    assert!(lines[2].starts_with("250"), "{}", lines[2]);

    client.write_all(b"QUIT\r\n").await.unwrap();
    drop(client);
    let _ = join.await;

    assert_eq!(
        mock.calls().len(),
        3,
        "handler called once per accepted RCPT"
    );
}

// ---------------------------------------------------------------------------
// Drain (graceful shutdown) — Bucket B
// ---------------------------------------------------------------------------

/// Graceful path: accept a session, signal shutdown, the in-flight session
/// completes cleanly via QUIT, serve_with_listener returns Ok, no aborts.
#[tokio::test]
async fn drain_lets_in_flight_session_finish() {
    let state = make_state();
    let mock: Arc<dyn DeliveryHandler> = Arc::new(MockHandler::with_verdicts([Verdict::Delivered]));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let serve_join = tokio::spawn(async move {
        serve_with_listener(
            state,
            listener,
            mock,
            async {
                let _ = shutdown_rx.await;
            },
            std::time::Duration::from_secs(5),
        )
        .await
    });

    // Open a session, complete it, signal shutdown.
    let mut sock = tokio::net::TcpStream::connect(addr).await.unwrap();
    let _ = read_n_lines(&mut sock, 1).await;
    sock.write_all(b"LHLO t\r\n").await.unwrap();
    let _ = read_n_lines(&mut sock, 3).await;
    sock.write_all(b"MAIL FROM:<a@a>\r\n").await.unwrap();
    let _ = read_n_lines(&mut sock, 1).await;
    sock.write_all(format!("RCPT TO:<rampart-1@{INTERNAL_DOMAIN}>\r\n").as_bytes())
        .await
        .unwrap();
    let _ = read_n_lines(&mut sock, 1).await;
    sock.write_all(b"DATA\r\n").await.unwrap();
    let _ = read_n_lines(&mut sock, 1).await;
    sock.write_all(b"From: x\r\n\r\nb\r\n.\r\n").await.unwrap();
    let resp = read_n_lines(&mut sock, 1).await;
    assert!(resp.starts_with("250"), "{resp}");
    sock.write_all(b"QUIT\r\n").await.unwrap();

    // Now signal shutdown.
    let _ = shutdown_tx.send(());

    // serve should return Ok within the drain window (5s).
    let serve_result = tokio::time::timeout(std::time::Duration::from_secs(5), serve_join)
        .await
        .expect("serve must return within drain window")
        .expect("join")
        .expect("serve_with_listener returned err");
    let _ = serve_result;
}

/// Drain timeout path: a session is still parked when shutdown fires
/// AND the drain window expires; serve still returns Ok and aborts the
/// remaining session.
#[tokio::test]
async fn drain_aborts_after_timeout() {
    let state = make_state();
    let mock: Arc<dyn DeliveryHandler> = Arc::new(MockHandler::with_verdicts([]));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let serve_join = tokio::spawn(async move {
        serve_with_listener(
            state,
            listener,
            mock,
            async {
                let _ = shutdown_rx.await;
            },
            // Short drain — the session below is parked waiting for our
            // input that we never send.
            std::time::Duration::from_millis(200),
        )
        .await
    });

    // Connect but don't drive the session anywhere; it sits idle inside
    // the read loop. Shutdown is signalled while it's parked.
    let mut sock = tokio::net::TcpStream::connect(addr).await.unwrap();
    let _ = read_n_lines(&mut sock, 1).await;
    let _ = shutdown_tx.send(());

    // serve_with_listener should return Ok within (drain + slack). The
    // parked session gets abort_all'd; that's the "aborts" branch.
    let serve_result = tokio::time::timeout(std::time::Duration::from_secs(2), serve_join)
        .await
        .expect("serve must return within drain + slack")
        .expect("join")
        .expect("serve_with_listener returned err");
    let _ = serve_result;
    drop(sock);
}
