# Hadrone

A responsive, draggable/resizable grid system for Rust, inspired by [react-grid-layout](https://github.com/react-grid-layout/react-grid-layout). The spatial engine is deterministic headless Rust (`hadrone-core`); UI integrations cover **Dioxus 0.7**, **Leptos**, **Yew**, and **vanilla JS** via **WebAssembly**.

## Key features

- **Shared engine**: Compaction, collision handling, pointer/session math, and optional validation live in `hadrone-core` (no DOM types in core).
- **Framework crates**: `hadrone-dioxus`, `hadrone-leptos`, `hadrone-yew` with aligned drag/resize behavior; **Dioxus** also exposes structured **`LayoutEvent`** lifecycle hooks (`on_layout_event`, `emit_interaction_updates`).
- **Compaction**: `CompactionType::Gravity` (vertical “rising tide”) and `FreePlacement`.
- **Collisions**: Pluggable **`CollisionStrategy`** (`PushDown` default, `None` to skip displacement until compaction).
- **Layout model**: `LayoutItem` supports min/max size, optional **aspect ratio** (width/height), **`is_static`**, and separate **`is_draggable` / `is_resizable`** (RGL-style).
- **Import/export**: `validate_layout`, `repair_layout`, and responsive **`scale_layout_cols`** / **`select_breakpoint`** for breakpoint changes.
- **Extras**: `hadrone-extras` provides storage adapters and responsive breakpoint helpers (e.g. web + Dioxus).

## Architecture

| Crate | Role |
|--------|------|
| `hadrone-core` | Grid math, `LayoutEngine`, `InteractionSession`, validation, responsive helpers, `LayoutEvent` types |
| `hadrone-dioxus` | `GridLayout` / `GridItem` for Dioxus 0.7 |
| `hadrone-leptos` | `GridLayout` for Leptos |
| `hadrone-yew` | `GridLayout` for Yew |
| `hadrone-wasm` | JS bindings (`GridEngineWasm`) for vanilla front ends |
| `hadrone-extras` | Persistence (`LayoutStorage`), `BreakpointConfig`, debounced save |
| `packages/ui` | Shared dashboard CSS and UI helpers used by web/desktop |

## Getting started

### Dioxus (dashboards and `packages/web`)

```bash
cargo run -p dioxus-dashboard              # desktop dashboard example
dx serve --package packages/web           # web Grid Engine Debugger
dx serve --package packages/desktop       # desktop shell with file-backed layout
```

### Leptos

```bash
cargo run --example leptos-dashboard
```

### Yew

```bash
cargo run --example yew-dashboard
```

### Vanilla JS (WASM)

Build bindings into `examples/vanilla-js/pkg/`, then serve **only** `examples/vanilla-js` (see `examples/vanilla-js/README.md`):

```bash
cargo run -p vanilla-js-runner            # wasm-pack build + prints serve instructions
cd examples/vanilla-js && python3 -m http.server 8083
```

Open `http://localhost:8083`.

## Minimal usage (Dioxus)

`GridLayout` requires `margin`, `compaction`, and `render_item` (other props have defaults such as `container_padding`, `collision_strategy`).

```rust
use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem};
use hadrone_dioxus::GridLayout;

#[component]
fn Dashboard() -> Element {
    let mut layout = use_signal(|| vec![
        LayoutItem { id: "1".into(), x: 0, y: 0, w: 4, h: 2, ..Default::default() },
    ]);

    rsx! {
        GridLayout {
            layout,
            cols: 12,
            row_height: 100.0,
            margin: (10, 10),
            compaction: CompactionType::Gravity,
            render_item: |item: LayoutItem| rsx! {
                div { class: "widget", "Widget {item.id}" }
            },
        }
    }
}
```

## Minimal usage (Leptos)

```rust
use hadrone_core::CompactionType;
use hadrone_leptos::GridLayout;
use leptos::*;

#[component]
fn Dashboard() -> impl IntoView {
    let layout = create_rw_signal(vec![]);

    view! {
        <GridLayout
            layout=layout
            cols=12.into()
            row_height=100.0.into()
            margin=(10, 10).into()
            compaction=CompactionType::Gravity.into()
            render_item=move |item| view! { <div class="widget">"Widget " {item.id}</div> }
        />
    }
}
```

## Interaction model

During drag/resize, the core **`InteractionSession`** tracks sub-pixel movement and snaps grid updates from deltas, so the live element can stay smooth while the committed layout stays grid-aligned. **`container_padding`** on the session matches CSS padding on the grid container when you need pixel-aligned overlays.

## License

MIT / Apache-2.0
