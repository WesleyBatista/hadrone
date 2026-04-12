# UI package (`packages/ui`)

Shared styling and small UI fragments for **web** and **desktop** Hadrone demos.

## Contents

- **`assets/styling/hadrone-dashboard.css`** — Shell layout (`.hadrone-dashboard`, header, controls, grid panel) and widget cards (`.hadrone-widget`, …). Consumed by Dioxus apps via the `ui` crate; the **vanilla JS** example vendors a copy under `examples/vanilla-js/assets/` so a simple static server works.
- **`assets/styling/navbar.css`** — Optional nav styling.
- **`src/`** — Rust helpers (`dashboard_styles`, `navbar`) for embedding or referencing assets from Dioxus.

Keep this crate free of framework-specific runtime dependencies beyond what `Cargo.toml` already uses; platform APIs belong in `packages/web` or `packages/desktop`.
