use crate::grid_helpers::*;
use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use hadrone_extras::{FileStorage, LayoutSnapshot, LayoutStorage};
use std::collections::HashSet;
use std::rc::Rc;

pub const RSX_CODE: &str = r#"GridLayout {
    layout,
    cols: 12,
    row_height: 120.0,
    margin: (20, 20),
    compaction: CompactionType::FreePlacement,
    keyboard_cell_nudge: true,
    render_item: |item: LayoutItem| default_widget(&item),
}"#;

fn default_layout() -> Vec<LayoutItem> {
    let mut handles = HashSet::new();
    handles.insert(ResizeHandle::SouthEast);
    handles.insert(ResizeHandle::South);
    handles.insert(ResizeHandle::East);

    vec![
        LayoutItem {
            id: "weather".into(),
            x: 0,
            y: 0,
            w: 4,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "analytics".into(),
            x: 4,
            y: 0,
            w: 6,
            h: 4,
            resize_handles: handles.clone(),
            aspect_ratio: Some(1.5),
            ..Default::default()
        },
        LayoutItem {
            id: "stock".into(),
            x: 0,
            y: 2,
            w: 4,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
    ]
}

#[component]
pub fn DebuggerExample() -> Element {
    let mut layout = use_signal(default_layout);
    let mut cols = use_signal(|| 12);
    let storage = Rc::new(FileStorage::new("./storage"));

    let storage_load = storage.clone();
    use_effect(move || {
        if let Ok(Some(snapshot)) = storage_load.load("dashboard_v2") {
            layout.set(snapshot.items);
            if snapshot.cols > 0 {
                cols.set(snapshot.cols);
            }
        }
    });

    let storage_save = storage.clone();

    let on_reset = move |_| {
        layout.set(default_layout());
        cols.set(12);
    };

    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "Grid Engine Debugger",
                description: "Full-featured dashboard demonstrating all hadrone-core capabilities: drag, resize, aspect ratio locking, and more.",
                show_code: true,
                code: Some(RSX_CODE),
                show_reset: true,
                on_reset: EventHandler::new(on_reset),
            }

            ExampleControls {
                ControlGroup { label: "Columns",
                    input {
                        r#type: "range",
                        min: "1",
                        max: "24",
                        value: "{cols()}",
                        style: "width: 150px; cursor: pointer;",
                        oninput: move |e| {
                            let val = e.value().parse::<i32>().unwrap_or(12);
                            cols.set(val);
                        }
                    }
                    span { style: "font-size: 12px; color: #64748b;", "{cols()} cols" }
                }

                ControlGroup { label: "Actions",
                    ControlButton {
                        label: "Save Layout",
                        secondary: false,
                        onclick: move |_| {
                            let _ = storage_save.save("dashboard_v2", &LayoutSnapshot {
                                version: 1,
                                items: layout.peek().clone(),
                                cols: cols(),
                            });
                        }
                    }
                }
            }

            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: cols(),
                    row_height: 120.0,
                    margin: (20, 20),
                    compaction: CompactionType::FreePlacement,
                    keyboard_cell_nudge: true,
                    render_item: |item: LayoutItem| default_widget(&item),
                    on_layout_change: move |_| {}
                }
            }

            div { style: "margin-top: 24px; padding: 16px; background: #fef3c7; border-radius: 12px; border: 1px solid #f59e0b;",
                strong { style: "color: #92400e;", "Tip: " }
                span { style: "color: #78350f;", "Drag and resize widgets to rearrange the dashboard. The analytics widget has aspect ratio locking (1.5:1)." }
            }
        }
    }
}
