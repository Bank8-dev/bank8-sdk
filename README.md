# bank8-sdk — Bank8 Namespace Facade

## 1. Summary

The Bank8 vertical's SDK workspace: one cargo workspace holding the
vertical's contracts crates and the `bank8` facade crate. Every
bank8 service (and external consumer) imports through the facade —
`use bank8::ledger::contracts::…` (namespace rule,
architecture-principles §2.1). Zero platform knowledge leaks the
other way: `sentin8` never references bank8 (§4.14).

## 2. Crates

| Crate | Purpose |
|---|---|
| `facade` (package `bank8`) | The namespace facade — every leaf crate nests under its service module |
| `crates/ledger/contracts` | Bank8:Ledger wire types — accounts, postings, statements |
| `crates/payments/contracts` | Bank8:Payments wire types — instructions, batches, calendars |
| `crates/fdx/contracts` | Bank8:FDX wire types — consent-driven financial data exchange |
| `crates/regulatory/contracts` | Shared jurisdiction registry + report kinds (reporting/risk/compliance) |

## 3. Consumption

```toml
bank8 = { git = "https://github.com/Bank8-dev/bank8-sdk", tag = "bank8-v0.1.0" }
```

The facade pins the platform SDK (`sentin8` — shared plane only,
SDD §2.2); consumers inherit it transitively and never need a second
platform pin.

---

**Design attribution:** bank8:sdk and its underlying concepts were designed by Hamid Jahanzad <Hamid@Jahanzad.com>.
