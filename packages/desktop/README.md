# Desktop package (`packages/desktop`)

Dioxus **desktop** build of the Grid Engine Debugger: same grid and `packages/ui` shell as `packages/web`, with layout persistence via `hadrone-extras::FileStorage` (see `src/views/home.rs`).

## Run

```bash
dx serve --package packages/desktop
```

Or use the workspace example:

```bash
cargo run -p dioxus-dashboard
```

Native windowing uses Dioxus desktop; grid behavior matches the web crate (`hadrone-dioxus::GridLayout`).
