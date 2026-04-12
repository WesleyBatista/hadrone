use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use hadrone_extras::{
    use_responsive_grid, BreakpointConfig, BrowserStorage, LayoutSnapshot, LayoutStorage,
};
use std::collections::HashSet;
use std::rc::Rc;

#[component]
pub fn DashboardHome() -> Element {
    let current_bp = use_responsive_grid(vec![
        BreakpointConfig {
            name: "mobile".into(),
            cols: 4,
            min_width: 0,
            margin: (10, 10),
            row_height: 80.0,
        },
        BreakpointConfig {
            name: "tablet".into(),
            cols: 8,
            min_width: 640,
            margin: (15, 15),
            row_height: 90.0,
        },
        BreakpointConfig {
            name: "desktop".into(),
            cols: 12,
            min_width: 1024,
            margin: (20, 20),
            row_height: 100.0,
        },
    ]);

    let mut manual_cols = use_signal(|| None::<i32>);
    let effective_cols = manual_cols().unwrap_or(current_bp.read().cols);

    let mut layout = use_signal(|| {
        let mut handles = HashSet::new();
        handles.insert(ResizeHandle::SouthEast);
        handles.insert(ResizeHandle::South);
        handles.insert(ResizeHandle::East);

        vec![
            LayoutItem {
                id: "1".into(),
                x: 0,
                y: 0,
                w: 4,
                h: 2,
                resize_handles: handles.clone(),
                ..Default::default()
            },
            LayoutItem {
                id: "2".into(),
                x: 4,
                y: 0,
                w: 4,
                h: 4,
                resize_handles: handles.clone(),
                aspect_ratio: Some(1.0),
                ..Default::default()
            },
            LayoutItem {
                id: "3".into(),
                x: 2,
                y: 2,
                w: 2,
                h: 2,
                resize_handles: handles.clone(),
                ..Default::default()
            },
        ]
    });

    let storage = Rc::new(BrowserStorage::new("test-rg-web"));

    let storage_load = storage.clone();
    use_effect(move || {
        if let Ok(Some(snapshot)) = storage_load.load("web_dashboard_v1") {
            layout.set(snapshot.items);
            if snapshot.cols > 0 {
                manual_cols.set(Some(snapshot.cols));
            }
        }
    });

    let storage_save = storage.clone();

    rsx! {
        div {
            class: "hadrone-dashboard",
            header { class: "hadrone-dashboard__header",
                div { class: "hadrone-dashboard__title-block",
                    h1 { class: "hadrone-dashboard__title", "Grid Engine Debugger" }
                    p { class: "hadrone-dashboard__subtitle",
                        "Web build: responsive breakpoints adjust columns, row height, and margins. Layout can be saved to browser storage. Desktop build uses the same shell with fixed geometry and file persistence—see the desktop package."
                    }
                    div { class: "hadrone-dashboard__meta",
                        span { "Platform: Web" }
                        span { class: "hadrone-dashboard__meta-sep", "•" }
                        span { "BP: {current_bp.read().name}" }
                        span { class: "hadrone-dashboard__meta-sep", "•" }
                        span { "Cols: {effective_cols}" }
                        span { class: "hadrone-dashboard__meta-sep", "•" }
                        span { "Row: {current_bp.read().row_height}px" }
                        span { class: "hadrone-dashboard__meta-sep", "•" }
                        span { "Gap: {current_bp.read().margin.0}×{current_bp.read().margin.1}px" }
                    }
                }

                div { class: "hadrone-dashboard__controls",
                    div { class: "hadrone-dashboard__field",
                        label { class: "hadrone-dashboard__label", "Manual grid columns" }
                        input {
                            class: "hadrone-dashboard__range",
                            r#type: "range",
                            min: "1",
                            max: "24",
                            value: "{effective_cols}",
                            oninput: move |e| {
                                let val = e.value().parse::<i32>().unwrap_or(12);
                                manual_cols.set(Some(val));
                            },
                        }
                    }

                    button {
                        class: "hadrone-dashboard__btn-primary",
                        onclick: move |_| {
                            let _ = storage_save.save(
                                "web_dashboard_v1",
                                &LayoutSnapshot {
                                    version: 1,
                                    items: layout.peek().clone(),
                                    cols: effective_cols,
                                },
                            );
                        },
                        "Save layout"
                    }
                }
            }

            div { class: "hadrone-dashboard__grid-panel",
                GridLayout {
                    layout,
                    cols: effective_cols,
                    row_height: current_bp.read().row_height,
                    margin: current_bp.read().margin,
                    compaction: CompactionType::Gravity,
                    keyboard_cell_nudge: true,
                    render_item: |item: LayoutItem| rsx! {
                        div { class: "hadrone-widget",
                            div { class: "hadrone-widget__chrome",
                                span { "WIDGET {item.id.to_uppercase()}" }
                                if item.aspect_ratio.is_some() {
                                    span { class: "hadrone-widget__badge", "ASPECT" }
                                }
                            }
                            div { class: "hadrone-widget__body",
                                div { class: "hadrone-widget__stats",
                                    div { "LOC: {item.x},{item.y}" }
                                    div { "DIM: {item.w}×{item.h}" }
                                }
                                div { class: "hadrone-widget__placeholder",
                                    "Focus widget • arrows move cell"
                                }
                            }
                        }
                    },
                    on_layout_change: move |new_layout: Vec<LayoutItem>| {
                        println!("Layout sync: {} items", new_layout.len());
                    },
                }
            }
        }
    }
}
