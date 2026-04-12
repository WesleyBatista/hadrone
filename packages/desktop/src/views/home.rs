use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use hadrone_extras::{FileStorage, LayoutSnapshot, LayoutStorage};
use std::collections::HashSet;
use std::rc::Rc;

#[component]
pub fn DashboardHome() -> Element {
    const ROW_H: f32 = 100.0;
    const MARGIN: (i32, i32) = (20, 20);

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

    let mut cols = use_signal(|| 12);
    let storage = Rc::new(FileStorage::new("./storage"));

    let storage_load = storage.clone();
    use_effect(move || {
        if let Ok(Some(snapshot)) = storage_load.load("main_dashboard") {
            layout.set(snapshot.items);
            if snapshot.cols > 0 {
                cols.set(snapshot.cols);
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
                        "Desktop build: same default widget layout as the web demo. Columns are adjusted with the slider (mirroring the web manual column control). Layout persists to ./storage via FileStorage; the web app uses breakpoints and BrowserStorage instead."
                    }
                    div { class: "hadrone-dashboard__meta",
                        span { "Platform: Desktop" }
                        span { class: "hadrone-dashboard__meta-sep", "•" }
                        span { "Cols: {cols}" }
                        span { class: "hadrone-dashboard__meta-sep", "•" }
                        span { "Row: {ROW_H}px" }
                        span { class: "hadrone-dashboard__meta-sep", "•" }
                        span { "Gap: {MARGIN.0}×{MARGIN.1}px" }
                    }
                }

                div { class: "hadrone-dashboard__controls",
                    div { class: "hadrone-dashboard__field",
                        label { class: "hadrone-dashboard__label",
                            "Grid columns ({cols})"
                        }
                        input {
                            class: "hadrone-dashboard__range",
                            r#type: "range",
                            min: "1",
                            max: "24",
                            value: "{cols}",
                            oninput: move |e| {
                                let val = e.value().parse::<i32>().unwrap_or(12);
                                cols.set(val);
                            },
                        }
                    }

                    button {
                        class: "hadrone-dashboard__btn-primary",
                        onclick: move |_| {
                            let _ = storage_save.save(
                                "main_dashboard",
                                &LayoutSnapshot {
                                    version: 1,
                                    items: layout.peek().clone(),
                                    cols: cols(),
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
                    cols: cols(),
                    row_height: ROW_H,
                    margin: MARGIN,
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
