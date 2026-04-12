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
    compaction: CompactionType::FreePlacement,
    collision_strategy: CollisionStrategy::PushDown,  // Default!
    render_item: |item: LayoutItem| { /* ... */ },
}

// The grid engine automatically detects overlaps
// and resolves collisions by pushing items down."#;

fn get_collision_color(id: &str) -> &'static str {
    match id {
        "A" => "#ef4444",
        "B" => "#f97316",
        "C" => "#22c55e",
        "D" => "#3b82f6",
        "E" => "#8b5cf6",
        _ => "#6b7280",
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
            w: 3,
            h: 3,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "B".into(),
            x: 3,
            y: 0,
            w: 3,
            h: 3,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "C".into(),
            x: 6,
            y: 0,
            w: 3,
            h: 3,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "D".into(),
            x: 0,
            y: 3,
            w: 4,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "E".into(),
            x: 4,
            y: 3,
            w: 4,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
    ]
}

#[component]
pub fn CollisionsExample() -> Element {
    let layout = use_signal(|| default_layout());

    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "Collision Handling",
                description: "Items that would overlap are automatically adjusted. Try dragging items over each other to see how collisions are resolved.",
                show_code: true,
                code: Some(RSX_CODE),
                show_reset: false,
            }

            div { style: "margin-bottom: 24px; padding: 16px; background: #fce7f3; border-radius: 12px; border: 1px solid #ec4899;",
                strong { style: "color: #be185d;", "Collision Detection: " }
                span { style: "color: #831843;", "When you drag an item over another, the engine detects the collision and prevents overlap." }
            }

            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: 12,
                    row_height: 80.0,
                    margin: (10, 10),
                    compaction: CompactionType::FreePlacement,
                    render_item: |item: LayoutItem| {
                        let color = get_collision_color(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: {color}; border-radius: 8px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: white; font-weight: 700; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                span { style: "font-size: 28px;", "{item.id}" }
                                span { style: "font-size: 11px; opacity: 0.8;", "({item.x}, {item.y})" }
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
        }
    }
}
