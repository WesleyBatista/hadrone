use crate::grid_helpers::*;
use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use std::collections::HashSet;

pub const RSX_CODE: &str = r#"GridLayout {
    layout,
    cols: 12,
    row_height: 80.0,
    margin: (10, 10),
    compaction: CompactionType::Gravity,  // Key property!
    render_item: |item: LayoutItem| {
        // Items will automatically fall down
    },
}"#;

fn get_color(id: &str) -> &'static str {
    match id {
        "A" => "#ef4444",
        "B" => "#f97316",
        "C" => "#22c55e",
        "D" => "#3b82f6",
        _ => "#8b5cf6",
    }
}

fn default_layout() -> Vec<LayoutItem> {
    let mut handles = HashSet::new();
    handles.insert(ResizeHandle::SouthEast);

    vec![
        LayoutItem {
            id: "A".into(),
            x: 0,
            y: 0,
            w: 4,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "B".into(),
            x: 4,
            y: 0,
            w: 4,
            h: 3,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "C".into(),
            x: 0,
            y: 2,
            w: 4,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "D".into(),
            x: 8,
            y: 0,
            w: 4,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
    ]
}

#[component]
pub fn GravityExample() -> Element {
    let layout = use_signal(|| default_layout());

    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "Gravity Compaction",
                description: "Items fall down to fill empty space below them. This is similar to how masonry layouts work.",
                show_code: true,
                code: Some(RSX_CODE),
                show_reset: false,
            }

            div { style: "margin-bottom: 24px; padding: 16px; background: #d1fae5; border-radius: 12px; border: 1px solid #10b981;",
                strong { style: "color: #047857;", "Gravity Active: " }
                span { style: "color: #065f46;", "Items automatically compact vertically. Move items to create gaps and watch them fall." }
            }

            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: 12,
                    row_height: 80.0,
                    margin: (10, 10),
                    compaction: CompactionType::Gravity,
                    render_item: |item: LayoutItem| {
                        let color = get_color(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: {color}; border-radius: 8px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: white; font-weight: 700; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                span { style: "font-size: 24px;", "{item.id}" }
                                span { style: "font-size: 11px; opacity: 0.8;", "y: {item.y}" }
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
        }
    }
}
