use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use std::collections::HashSet;
use crate::grid_helpers::*;

pub const RSX_CODE: &str = r#"LayoutItem {
    id: "A".into(),
    x: 0,
    y: 0,
    w: 3,
    h: 2,
    resize_handles: handles.clone(),
    is_static: true,  // Key property!
    ..Default::default()
}

GridLayout {
    layout,
    cols: 12,
    compaction: CompactionType::FreePlacement,
    render_item: |item: LayoutItem| { /* ... */ },
}"#;

fn get_color(id: &str) -> &'static str {
    match id {
        "A" => "#ef4444",
        "B" => "#f97316",
        "C" => "#eab308",
        "D" => "#22c55e",
        "E" => "#3b82f6",
        _ => "#8b5cf6",
    }
}

fn default_layout() -> Vec<LayoutItem> {
    let mut handles = HashSet::new();
    handles.insert(ResizeHandle::SouthEast);
    
    vec![
        LayoutItem { id: "A".into(), x: 0, y: 0, w: 3, h: 2, resize_handles: handles.clone(), is_static: true, ..Default::default() },
        LayoutItem { id: "B".into(), x: 3, y: 0, w: 3, h: 2, resize_handles: handles.clone(), is_static: true, ..Default::default() },
        LayoutItem { id: "C".into(), x: 6, y: 0, w: 3, h: 2, resize_handles: handles.clone(), is_static: true, ..Default::default() },
        LayoutItem { id: "D".into(), x: 0, y: 2, w: 4, h: 3, resize_handles: handles.clone(), is_static: true, ..Default::default() },
        LayoutItem { id: "E".into(), x: 4, y: 2, w: 4, h: 3, resize_handles: handles.clone(), is_static: true, ..Default::default() },
        LayoutItem { id: "F".into(), x: 8, y: 2, w: 4, h: 3, resize_handles: handles.clone(), is_static: true, ..Default::default() },
    ]
}

#[component]
pub fn NoDraggingExample() -> Element {
    let layout = use_signal(|| default_layout());
    
    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "No Dragging (Static Layout)",
                description: "A read-only grid where all items are marked as static. Items cannot be dragged or resized.",
                show_code: true,
                code: Some(RSX_CODE),
                show_reset: false,
            }
            
            div { style: "margin-bottom: 24px; padding: 16px; background: #dbeafe; border-radius: 12px; border: 1px solid #3b82f6;",
                strong { style: "color: #1e40af;", "Static Mode: " }
                span { style: "color: #1e3a8a;", "All widgets have is_static: true. You can observe the layout but cannot modify it." }
            }
            
            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: 12,
                    row_height: 80.0,
                    margin: (10, 10),
                    compaction: CompactionType::FreePlacement,
                    render_item: |item: LayoutItem| {
                        let color = get_color(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: {color}; border-radius: 8px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: white; font-weight: 700; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                span { style: "font-size: 32px;", "{item.id}" }
                                span { style: "font-size: 12px; opacity: 0.8;", "Static" }
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
        }
    }
}
