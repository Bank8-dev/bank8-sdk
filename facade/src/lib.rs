// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # bank8 — Bank8 workspace namespace facade
//! Every bank8 crate is named `bank8::{leaf}` and is imported through this
//! facade; service-scoped crates nest under their service module. The SDK
//! client for each service lives in the same module (architecture-principles
//! §2.1; the SDK modules moved here from the platform SDK — zero vertical
//! knowledge in sentin8, §4.14).

/// Bank8:Ledger — SDK client + contracts.
pub mod ledger {
    pub use bank8_ledger_contracts as contracts;
    mod sdk;
    pub use sdk::{LedgerClient, LedgerError};
}

/// Bank8:Payments — SDK client + contracts.
pub mod payments {
    pub use bank8_payments_contracts as contracts;
    mod sdk;
    pub use sdk::{PaymentsClient, PaymentsError};
}

/// Bank8:FDX — SDK client + contracts.
pub mod fdx {
    pub use bank8_fdx_contracts as contracts;
    mod sdk;
    pub use sdk::{Fdx8Client, Fdx8Error};
}

/// Bank8 regulatory services (reporting, risk, compliance) — shared
/// jurisdiction registry and report-kind contracts, imported through the
/// facade per the namespace rule (architecture-principles §2.1).
pub mod regulatory {
    pub use bank8_regulatory_contracts as contracts;
}

/// Shared builder trait for all Sentin8 service clients.
pub trait ClientBuilder: Sized {
    /// Set the service base URL (trailing slash is stripped).
    fn with_base_url(self, url: impl Into<String>) -> Self;
    /// Attach a bearer auth token used for every request.
    fn with_token(self, token: impl Into<String>) -> Self;
    /// Provide a custom `reqwest::Client`.
    fn with_http_client(self, client: reqwest::Client) -> Self;
}

/// HTTP header used to propagate the correlation ID across services.
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// Generate a new correlation ID (UUID v4).
pub fn generate_correlation_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub(crate) fn prepare_request(
    http: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    token: Option<&str>,
) -> reqwest::RequestBuilder {
    let req = http.request(method, url);
    let req = req.header(CORRELATION_ID_HEADER, generate_correlation_id());
    if let Some(t) = token { req.bearer_auth(t) } else { req }
}

#[cfg(test)]
pub(crate) fn install_test_provider() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = sentin8::tls::crypto::sentin8_pq_provider().install_default();
    });
}
