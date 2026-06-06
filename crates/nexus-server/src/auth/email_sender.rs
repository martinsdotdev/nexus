//! The `EmailSender` port (ADR-0010). Delivering the one-time code is an impure side
//! effect at the edge of the system, so it lives behind a small enum the routes call.
//!
//! `Resend` is the real provider: a single HTTPS POST to api.resend.com, selected when
//! `NEXUS_RESEND_API_KEY` is set. Two interim variants remain for local/dev use: `Log`
//! writes the code to the tracing log, and `File` appends each `<recipient>\t<code>` line
//! to a file (a simple local sink, and what the auth e2e reads to complete a sign-in
//! without a mailbox). A `cfg(test)` `Capture` variant lets in-crate route tests read it.

use std::path::PathBuf;

/// How the relay delivers an email one-time code.
#[derive(Clone)]
pub enum EmailSender {
    /// Interim delivery: log the code (no email provider is wired yet).
    Log,
    /// Append each `<recipient>\t<code>` line to a file (a local sink; the e2e reads it).
    File(PathBuf),
    /// Real delivery via Resend (api.resend.com): a shared client plus the API key and the
    /// verified `from` address. Selected in `main` when `NEXUS_RESEND_API_KEY` is set.
    Resend {
        client: reqwest::Client,
        api_key: String,
        from: String,
    },
    /// Test-only: record every `(recipient, code)` so a route test can read it back.
    #[cfg(test)]
    Capture(std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>),
}

/// The Resend send-email request body (serialized to JSON by reqwest).
#[derive(serde::Serialize)]
struct ResendEmail<'a> {
    from: &'a str,
    to: [&'a str; 1],
    subject: &'a str,
    text: String,
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
            EmailSender::Resend {
                client,
                api_key,
                from,
            } => {
                let email = ResendEmail {
                    from,
                    to: [to],
                    subject: "Your Nexus sign-in code",
                    text: format!(
                        "Your Nexus sign-in code is {code}.\n\nIt expires in an hour. \
                         If you didn't try to sign in, you can ignore this email."
                    ),
                };
                let res = client
                    .post("https://api.resend.com/emails")
                    .bearer_auth(api_key)
                    .json(&email)
                    .send()
                    .await?;
                let status = res.status();
                if !status.is_success() {
                    let detail = res.text().await.unwrap_or_default();
                    anyhow::bail!("Resend API returned {status}: {detail}");
                }
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
