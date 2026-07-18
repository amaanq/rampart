//! Minimal LMTP listener. LHLO instead of EHLO, one response per RCPT
//! after DATA. Per-RCPT verdicts come from the pipeline.

use std::{
   sync::Arc,
   time::Duration,
};

use anyhow::Result;
use tokio::{
   io::{
      AsyncRead,
      AsyncReadExt,
      AsyncWrite,
      AsyncWriteExt,
      BufReader,
   },
   net::TcpListener,
   task::JoinSet,
};

use crate::worker::{
   DeliveryHandler,
   PipelineHandler,
   WorkerState,
   loop_guard,
   pipeline,
};

pub const INTERNAL_DOMAIN: &str = "internal.rampart.lmtp";
pub const MAX_MESSAGE_SIZE: usize = 50 * 1024 * 1024; // 50 MiB
pub const MAX_LINE_LEN: usize = 64 * 1024;

pub async fn serve(state: WorkerState) -> Result<()> {
   let addr = state.config.lmtp_listen;
   let listener = TcpListener::bind(addr).await?;
   tracing::info!(%addr, "rampart-worker LMTP listening");
   let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Ready]);

   let drain_timeout = Duration::from_secs(state.config.lmtp_drain_secs);
   let handler: Arc<dyn DeliveryHandler> = Arc::new(PipelineHandler);
   serve_with_listener(state, listener, handler, shutdown_signal(), drain_timeout).await
}

/// Test-friendly entry — caller supplies listener + handler + shutdown future.
pub async fn serve_with_listener<S>(
   state: WorkerState,
   listener: TcpListener,
   handler: Arc<dyn DeliveryHandler>,
   shutdown: S,
   drain_timeout: Duration,
) -> Result<()>
where
   S: std::future::Future<Output = ()> + Send,
{
   let mut handles: JoinSet<()> = JoinSet::new();
   let mut shutdown = std::pin::pin!(shutdown);

   loop {
      tokio::select! {
          biased;
          _ = &mut shutdown => break,
          accepted = listener.accept() => {
              let (sock, peer) = accepted?;
              let st = state.clone();
              let h = handler.clone();
              handles.spawn(async move {
                  let (r, w) = sock.into_split();
                  if let Err(e) =
                      handle_session_io(st, BufReader::new(r), w, h.as_ref()).await
                  {
                      tracing::error!(error = ?e, ?peer, "lmtp session ended with error");
                  }
              });
          }
      }
      // Reap to keep the JoinSet bounded.
      while handles.try_join_next().is_some() {}
   }

   let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Stopping]);
   drop(listener);
   tracing::info!(in_flight = handles.len(), "draining lmtp sessions");

   let drain = tokio::time::timeout(drain_timeout, async {
      while handles.join_next().await.is_some() {}
   })
   .await;
   if drain.is_err() {
      tracing::warn!(
         remaining = handles.len(),
         "drain timeout; aborting sessions"
      );
      handles.abort_all();
      while handles.join_next().await.is_some() {}
   }
   Ok(())
}

async fn shutdown_signal() {
   let ctrl_c = async {
      let _ = tokio::signal::ctrl_c().await;
   };
   #[cfg(unix)]
   let term = async {
      use tokio::signal::unix::{
         SignalKind,
         signal,
      };
      let mut s = signal(SignalKind::terminate()).expect("install SIGTERM");
      s.recv().await;
   };
   #[cfg(not(unix))]
   let term = std::future::pending::<()>();

   tokio::select! { _ = ctrl_c => {}, _ = term => {} }
}

/// Generic over r/w halves with a `DeliveryHandler` trait object so
/// DB-free state-machine tests can drive LMTP with a mock.
pub async fn handle_session_io<R, W>(
   state: WorkerState,
   mut reader: BufReader<R>,
   mut w: W,
   handler: &dyn DeliveryHandler,
) -> Result<()>
where
   R: AsyncRead + Unpin + Send,
   W: AsyncWrite + Unpin + Send,
{
   w.write_all(b"220 rampart-worker LMTP ready\r\n").await?;

   let mut mail_from: Option<String> = None;
   let mut rcpts: Vec<loop_guard::Rcpt> = Vec::new();
   let mut line_buf: Vec<u8> = Vec::with_capacity(256);

   loop {
      let n = read_line_bytes(&mut reader, &mut line_buf, MAX_LINE_LEN).await?;
      if n == 0 {
         break;
      }
      // Lossy-decode so non-UTF-8 doesn't panic the session.
      let line = String::from_utf8_lossy(&line_buf);
      let trimmed = line.trim_end_matches(['\r', '\n']);
      let upper = trimmed.to_ascii_uppercase();
      if upper.starts_with("LHLO ") || upper.starts_with("EHLO ") {
         w.write_all(b"250-rampart-worker\r\n250-PIPELINING\r\n250 8BITMIME\r\n")
            .await?;
      } else if upper.starts_with("MAIL FROM:") {
         let addr = parse_address_arg(trimmed);
         mail_from = Some(addr);
         rcpts.clear();
         w.write_all(b"250 OK\r\n").await?;
      } else if upper.starts_with("RCPT TO:") {
         let addr = parse_address_arg(trimmed);
         match loop_guard::parse_rcpt(&addr, INTERNAL_DOMAIN) {
            Some(r) => {
               rcpts.push(r);
               w.write_all(b"250 OK\r\n").await?;
            },
            None => {
               w.write_all(b"550 5.1.1 Not a routable rampart worker recipient\r\n")
                  .await?;
            },
         }
      } else if upper == "DATA" {
         if rcpts.is_empty() || mail_from.is_none() {
            w.write_all(b"503 5.5.1 Need MAIL and RCPT first\r\n")
               .await?;
            continue;
         }
         w.write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
            .await?;
         let mut body: Vec<u8> = Vec::new();
         let mut too_big = false;
         let mut saw_terminator = false;
         loop {
            let mut raw = Vec::with_capacity(1024);
            let n = read_line_bytes(&mut reader, &mut raw, MAX_LINE_LEN).await?;
            if n == 0 {
               break;
            }
            let line = raw.as_slice();
            let is_terminator = line == b".\r\n" || line == b".\n" || line == b".";
            if is_terminator {
               saw_terminator = true;
               break;
            }
            let slice = if line.starts_with(b"..") {
               &line[1..]
            } else {
               line
            };
            if body.len() + slice.len() > MAX_MESSAGE_SIZE {
               too_big = true;
            }
            if !too_big {
               body.extend_from_slice(slice);
            }
         }
         // EOF before the dot terminator = truncated DATA. Don't deliver:
         // upstream MTA never got a 250 and will retry, and we'd otherwise
         // submit a half-message that gets duplicated on the retry.
         if !saw_terminator {
            tracing::warn!(
               rcpts = rcpts.len(),
               body_bytes = body.len(),
               "lmtp DATA aborted before terminator; not delivering"
            );
            break;
         }
         if too_big {
            for _ in 0..rcpts.len() {
               w.write_all(b"552 5.3.4 Message exceeds size limit\r\n")
                  .await?;
            }
            rcpts.clear();
            mail_from = None;
            continue;
         }
         for rcpt in rcpts.drain(..) {
            let d = pipeline::Delivery {
               rcpt,
               mail_from: mail_from.clone().unwrap(),
               raw: body.clone(),
            };
            let verdict = handler.handle(&state, d).await;
            let reply = match verdict {
               pipeline::Verdict::Delivered => b"250 2.0.0 OK\r\n".to_vec(),
               pipeline::Verdict::Perm { internal, smtp } => {
                  tracing::info!(internal, smtp, "lmtp 550 perm");
                  format!("550 5.7.1 {smtp}\r\n").into_bytes()
               },
               pipeline::Verdict::Temp { internal, smtp } => {
                  tracing::info!(internal, smtp, "lmtp 451 temp");
                  format!("451 4.7.0 {smtp}\r\n").into_bytes()
               },
            };
            w.write_all(&reply).await?;
         }
         mail_from = None;
      } else if upper == "RSET" {
         mail_from = None;
         rcpts.clear();
         w.write_all(b"250 OK\r\n").await?;
      } else if upper == "NOOP" {
         w.write_all(b"250 OK\r\n").await?;
      } else if upper == "QUIT" {
         w.write_all(b"221 Bye\r\n").await?;
         break;
      } else if upper.is_empty() {
         continue;
      } else {
         w.write_all(b"500 5.5.2 Unknown command\r\n").await?;
      }
   }
   Ok(())
}

/// Strip "MAIL FROM:" / "RCPT TO:" framing and any trailing ESMTP params.
fn parse_address_arg(line: &str) -> String {
   let after = line.split_once(':').map(|(_, r)| r).unwrap_or(line);
   let after = after.trim();
   let start = after.find('<').map(|i| i + 1).unwrap_or(0);
   let end = after.find('>').unwrap_or(after.len());
   if end >= start {
      after[start..end].to_owned()
   } else {
      after.to_owned()
   }
}

/// One SMTP line (incl. trailing CRLF/LF) into `buf`; 0 on EOF.
/// Errors if longer than `max` (RFC 5321 caps at 1000 bytes; we allow 64 KiB).
async fn read_line_bytes<R: AsyncReadExt + Unpin>(
   reader: &mut BufReader<R>,
   buf: &mut Vec<u8>,
   max: usize,
) -> std::io::Result<usize> {
   buf.clear();
   loop {
      let mut byte = [0u8; 1];
      let n = reader.read(&mut byte).await?;
      if n == 0 {
         return Ok(buf.len());
      }
      buf.push(byte[0]);
      if byte[0] == b'\n' {
         return Ok(buf.len());
      }
      if buf.len() >= max {
         return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "line too long",
         ));
      }
   }
}
