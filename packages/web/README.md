# Web package (`packages/web`)

Dioxus 0.7 app: **Grid Engine Debugger** for the browser, with responsive breakpoints (`hadrone-extras::use_responsive_grid`), optional layout persistence to `localStorage`, and the shared dashboard shell from `packages/ui`.

## Run

From the workspace root:

```bash
dx serve --package packages/web
```

The main view lives in `src/views/home.rs` and composes `hadrone_dioxus::GridLayout` with the same breakpoint defaults as the vanilla JS example (`examples/vanilla-js`).

## Assets

- `assets/main.css` — minimal global styles; dashboard chrome comes from `packages/ui` (imported via the `ui` crate or asset paths in the app).
- Favicon and other static files live under `assets/` as needed for `dx serve`.

## Fullstack note

If you enable Dioxus fullstack later, keep web-only dependencies behind the `web` feature so server builds stay lean (see Dioxus fullstack docs).
