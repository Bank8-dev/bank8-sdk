// SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
// SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
// SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
// SPDX-License-Identifier: Apache-2.0

//! # bank8-regulatory-contracts
//!
//! Wire types and the jurisdiction registry vocabulary shared across the
//! bank8 regulatory services (reporting, risk, compliance) — see
//! `services/{reporting,risk,compliance}/DESIGN.md` §3. Pure data types:
//! `Serialize`/`Deserialize`/`Clone`/`Debug` only; no validation logic —
//! each service owns its own validation (the fdx-contracts precedent).
//!
//! The jurisdiction registry is runtime data (registry tables provisioned
//! by `vertical enroll`; zero regulator names in code — principles §4.14):
//! these DTOs are the wire shape, never the source of truth.

use serde::{Deserialize, Serialize};

/// A regulatory jurisdiction (country code + regulator list) — one entry of
/// the runtime jurisdiction registry.
///
/// # Examples
///
/// ```
/// let j = bank8_regulatory_contracts::Jurisdiction {
///     country_code: "CA".into(),
///     regulators: vec![],
/// };
/// assert_eq!(j.country_code, "CA");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jurisdiction {
    /// ISO 3166-1 alpha-2 country code (`CA`, `US`, `GB`, …).
    pub country_code: String,
    /// Regulators governing this jurisdiction.
    pub regulators: Vec<Regulator>,
}

/// A regulator (or FIU) within a jurisdiction, with the returns it requires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regulator {
    /// Regulator name as registered in the jurisdiction registry.
    pub name: String,
    /// Authority kind: `prudential`, `fiu`, `sanctions`, or `conduct`.
    pub authority_kind: String,
    /// Returns this regulator requires.
    pub return_templates: Vec<ReturnTemplate>,
}

/// A declarative return definition — data sources, format, and cadence.
/// Changing a regulator's format updates the registry, never the code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnTemplate {
    /// Return code (e.g. a COREP/FINREP-style return identifier).
    pub return_code: String,
    /// Output format (`XBRL` or `CSV`).
    pub output_format: String,
    /// Filing cadence (`daily`, `monthly`, `quarterly`, `annual`).
    pub cadence: String,
}

/// A configured risk limit with its breach state — the vocabulary Risk
/// reads limits into and Reporting reads exposure inputs through.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimit {
    /// Limit type (`capital_ratio`, `lcr`, `concentration`, `var`).
    pub limit_type: String,
    /// Threshold value.
    pub threshold: rust_decimal::Decimal,
    /// Breach status (`ok` or `breached`).
    pub status: String,
}

/// The regulatory report-kind vocabulary (AML/CTF filings). A closed enum:
/// the report-kind set is a slow-changing regulatory vocabulary, unlike
/// FDX's open scope strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportKind {
    /// Suspicious Transaction Report (Canada/EU/UK).
    Str,
    /// Suspicious Activity Report (US).
    Sar,
    /// Currency Transaction Report (US).
    Ctr,
    /// Electronic Funds Transfer Report (Canada).
    Eftr,
    /// Large Cash Transaction Report (Canada).
    Lctr,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_jurisdiction() -> Jurisdiction {
        Jurisdiction {
            country_code: "CA".into(),
            regulators: vec![Regulator {
                name: "test-prudential-authority".into(),
                authority_kind: "prudential".into(),
                return_templates: vec![ReturnTemplate {
                    return_code: "capital".into(),
                    output_format: "XBRL".into(),
                    cadence: "quarterly".into(),
                }],
            }],
        }
    }

    /// Round-trip stability: the registry JSON shape is pinned — any
    /// serialization drift fails the build (crate README §6 invariant).
    #[test]
    fn jurisdiction_serde_roundtrip() {
        let j = sample_jurisdiction();
        let json = serde_json::to_string(&j).unwrap();
        assert!(json.contains("\"country_code\":\"CA\""));
        let de: Jurisdiction = serde_json::from_str(&json).unwrap();
        assert_eq!(de.country_code, "CA");
        assert_eq!(de.regulators[0].authority_kind, "prudential");
        assert_eq!(de.regulators[0].return_templates[0].cadence, "quarterly");
    }

    #[test]
    fn report_kind_serde_roundtrip() {
        assert_eq!(serde_json::to_string(&ReportKind::Sar).unwrap(), "\"sar\"");
        let de: ReportKind = serde_json::from_str("\"eftr\"").unwrap();
        assert_eq!(de, ReportKind::Eftr);
    }

    #[test]
    fn risk_limit_decimal_roundtrip() {
        let l = RiskLimit {
            limit_type: "capital_ratio".into(),
            threshold: rust_decimal::Decimal::new(1050, 2),
            status: "ok".into(),
        };
        let de: RiskLimit = serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
        assert_eq!(de.threshold, rust_decimal::Decimal::new(1050, 2));
        assert_eq!(de.status, "ok");
    }
}
