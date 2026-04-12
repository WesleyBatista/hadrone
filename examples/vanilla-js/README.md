# Vanilla JS (WASM) example

This demo loads `hadrone-wasm` in the browser and mirrors the **Grid Engine Debugger** shell used by `packages/web` (same default widgets and shared dashboard styles).

## Build the WASM package

From the repo root (requires [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)):

```bash
cargo run -p vanilla-js-runner
```

That runs `wasm-pack build` for `crates/hadrone-wasm` and writes bindings to **`examples/vanilla-js/pkg/`**.

## Serve this folder

The static server **document root must be `examples/vanilla-js`**, because the page loads `./pkg/hadrone_wasm.js` and `./assets/hadrone-dashboard.css` as same-origin resources. Serving from the repo root would break those relative URLs unless you reconfigure paths.

```bash
cd examples/vanilla-js
python3 -m http.server 8083
```

Open `http://localhost:8083` (or `http://localhost:8083/index.html`).

## Styles

`assets/hadrone-dashboard.css` is a **vendored copy** of `packages/ui/assets/styling/hadrone-dashboard.css`. If you change the shared design tokens in `packages/ui`, update the vendored file (or switch to serving from the monorepo root with adjusted paths).

## Storage

Layout snapshots use the shape `{ "version": 1, "items": [...], "cols": null | number }`, aligned with `hadrone_extras::LayoutSnapshot` / the web dashboard.
