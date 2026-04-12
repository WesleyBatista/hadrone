use crate::grid_helpers::*;
use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use std::collections::HashSet;

pub const RSX_CODE: &str = r#"LayoutItem {
    id: "16:9 Video".into(),
    x: 0,
    y: 0,
    w: 4,
    h: 3,
    resize_handles: handles.clone(),
    aspect_ratio: Some(16.0 / 9.0),  // Key property!
    ..Default::default()
}

// Different aspect ratios:
// - 16.0 / 9.0 = 16:9 widescreen
// - 1.0 = 1:1 square
// - 4.0 / 3.0 = 4:3 photo
// - 2.0 = 2:1 banner"#;

fn get_aspect_color(id: &str) -> &'static str {
    if id.contains("16:9") {
        "#ef4444"
    } else if id.contains("1:1") {
        "#3b82f6"
    } else if id.contains("4:3") {
        "#22c55e"
    } else if id.contains("2:1") {
        "#8b5cf6"
    } else {
        "#6b7280"
    }
}

fn get_aspect_ratio_str(id: &str) -> &'static str {
    if id.contains("16:9") {
        "16:9"
    } else if id.contains("1:1") {
        "1:1"
    } else if id.contains("4:3") {
        "4:3"
    } else if id.contains("2:1") {
        "2:1"
    } else {
        "Free"
    }
}

fn default_layout() -> Vec<LayoutItem> {
    let mut handles = HashSet::new();
    handles.insert(ResizeHandle::SouthEast);
    handles.insert(ResizeHandle::South);
    handles.insert(ResizeHandle::East);

    vec![
        LayoutItem {
            id: "16:9 Video".into(),
            x: 0,
            y: 0,
            w: 4,
            h: 3,
            resize_handles: handles.clone(),
            aspect_ratio: Some(16.0 / 9.0),
            ..Default::default()
        },
        LayoutItem {
            id: "1:1 Square".into(),
            x: 4,
            y: 0,
            w: 3,
            h: 3,
            resize_handles: handles.clone(),
            aspect_ratio: Some(1.0),
            ..Default::default()
        },
        LayoutItem {
            id: "4:3 Photo".into(),
            x: 7,
            y: 0,
            w: 3,
            h: 4,
            resize_handles: handles.clone(),
            aspect_ratio: Some(4.0 / 3.0),
            ..Default::default()
        },
        LayoutItem {
            id: "2:1 Banner".into(),
            x: 0,
            y: 3,
            w: 4,
            h: 3,
            resize_handles: handles.clone(),
            aspect_ratio: Some(2.0),
            ..Default::default()
        },
        LayoutItem {
            id: "Unconstrained".into(),
            x: 4,
            y: 3,
            w: 3,
            h: 3,
            resize_handles: handles.clone(),
            ..Default::default()
        },
    ]
}

#[component]
pub fn AspectRatioExample() -> Element {
    let layout = use_signal(|| default_layout());

    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "Aspect Ratio Constraints",
                description: "Items maintain their pixel aspect ratio during resize. Try resizing each widget to see the constraint in action.",
                show_code: true,
                code: Some(RSX_CODE),
                show_reset: false,
            }

            div { style: "margin-bottom: 24px; padding: 16px; background: #fef3c7; border-radius: 12px; border: 1px solid #f59e0b;",
                strong { style: "color: #92400e;", "Aspect Ratio Lock: " }
                span { style: "color: #78350f;", "Each colored widget has a different aspect ratio constraint." }
            }

            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: 10,
                    row_height: 50.0,
                    margin: (10, 10),
                    compaction: CompactionType::FreePlacement,
                    render_item: |item: LayoutItem| {
                        let color = get_aspect_color(&item.id);
                        let ratio_str = get_aspect_ratio_str(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: {color}; border-radius: 8px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: white; font-weight: 700; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                span { style: "font-size: 14px;", "{item.id}" }
                                span { style: "font-size: 11px; opacity: 0.8;", "{ratio_str}" }
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
        }
    }
}
