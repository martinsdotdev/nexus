//! The `EmailSender` port (ADR-0010). Delivering the one-time code is an impure side
//! effect at the edge of the system, so it lives behind a small enum the routes call.
//!
//! No real provider is wired yet (ADR-0010 defers Resend/Postmark/SES), so the only
//! production variant is `Log`: it writes the code to the tracing log, which is how an
//! operator retrieves a code in the interim. A `cfg(test)` `Capture` variant lets route
//! tests read the code a handler "sent" without a real mailbox. Adding a real provider
//! later is just another variant.

/// How the relay delivers an email one-time code.
#[derive(Clone)]
pub enum EmailSender {
    /// Interim delivery: log the code (no email provider is wired yet).
    Log,
    /// Test-only: record every `(recipient, code)` so a route test can read it back.
    #[cfg(test)]
    Capture(std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>),
}

impl EmailSender {
    /// Deliver `code` to `to`. Fallible because a real provider can fail; the `Log`
    /// variant never does.
    pub async fn send_code(&self, to: &str, code: &str) -> anyhow::Result<()> {
        match self {
            EmailSender::Log => {
                tracing::info!(%to, %code, "email-code delivery (log-only; no provider wired)");
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
