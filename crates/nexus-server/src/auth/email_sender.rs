//! The `EmailSender` port (ADR-0010). Delivering the one-time code is an impure side
//! effect at the edge of the system, so it lives behind a small enum the routes call.
//!
//! No real provider is wired yet (ADR-0010 defers Resend/Postmark/SES). Two production
//! variants exist meanwhile: `Log` writes the code to the tracing log (how an operator
//! retrieves it in the interim), and `File` appends each `<recipient>\t<code>` line to a
//! file (a simple local sink, and what the auth e2e reads to complete a sign-in without a
//! mailbox). A `cfg(test)` `Capture` variant lets in-crate route tests read the code.
//! Adding a real provider later is just another variant.

use std::path::PathBuf;

/// How the relay delivers an email one-time code.
#[derive(Clone)]
pub enum EmailSender {
    /// Interim delivery: log the code (no email provider is wired yet).
    Log,
    /// Append each `<recipient>\t<code>` line to a file (a local sink; the e2e reads it).
    File(PathBuf),
    /// Test-only: record every `(recipient, code)` so a route test can read it back.
    #[cfg(test)]
    Capture(std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>),
}

impl EmailSender {
    /// Deliver `code` to `to`. Fallible because a real provider (or a file write) can
    /// fail; the `Log` variant never does.
    pub async fn send_code(&self, to: &str, code: &str) -> anyhow::Result<()> {
        match self {
            EmailSender::Log => {
                tracing::info!(%to, %code, "email-code delivery (log-only; no provider wired)");
                Ok(())
            }
            EmailSender::File(path) => {
                use std::io::Write as _;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                writeln!(file, "{to}\t{code}")?;
                Ok(())
            }
            #[cfg(test)]
            EmailSender::Capture(sent) => {
                sent.lock()
                    .expect("capture mutex is not poisoned")
                    .push((to.to_string(), code.to_string()));
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_file_sink_appends_each_recipient_and_code() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("codes.log");
        let sender = EmailSender::File(path.clone());

        sender.send_code("a@example.com", "ABCD2345").await.unwrap();
        sender.send_code("b@example.com", "WXYZ6789").await.unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("a@example.com\tABCD2345"));
        assert!(contents.contains("b@example.com\tWXYZ6789"));
    }
}
