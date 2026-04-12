use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use std::collections::HashSet;

const ITEM_COLORS: [&str; 8] = [
    "#f43f5e", "#f97316", "#eab308", "#22c55e", "#14b8a6", "#3b82f6", "#8b5cf6", "#ec4899",
];

fn get_item_color(id: &str) -> String {
    let idx = if id.starts_with('n') {
        id[1..].parse::<usize>().unwrap_or(0) % 8
    } else {
        id.parse::<usize>().unwrap_or(0) % 8
    };
    ITEM_COLORS[idx].to_string()
}

fn default_layout() -> Vec<LayoutItem> {
    let mut handles = HashSet::new();
    handles.insert(ResizeHandle::SouthEast);

    vec![
        LayoutItem {
            id: "0".into(),
            x: 0,
            y: 0,
            w: 2,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "1".into(),
            x: 2,
            y: 0,
            w: 2,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "2".into(),
            x: 4,
            y: 0,
            w: 2,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "3".into(),
            x: 6,
            y: 0,
            w: 2,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
        LayoutItem {
            id: "4".into(),
            x: 8,
            y: 0,
            w: 2,
            h: 2,
            resize_handles: handles.clone(),
            ..Default::default()
        },
    ]
}

#[component]
pub fn DynamicAddRemoveExample() -> Element {
    let mut layout = use_signal(|| default_layout());

    rsx! {
        div { class: "example-content",
            div { class: "example-header",
                h1 { class: "example-header__title", "Dynamic Layout Items" }
                p { class: "example-header__desc", "A grid with multiple colored items. Each item has a unique color based on its ID. Drag and resize items freely." }

                div { class: "example-header__actions",
                    button {
                        class: "example-header__action-btn",
                        onclick: move |_| layout.set(default_layout()),
                        "↺ Reset Layout"
                    }
                }
            }

            div { style: "margin-bottom: 24px; padding: 16px; background: #fef3c7; border-radius: 12px; border: 1px solid #f59e0b;",
                strong { style: "color: #92400e;", "Note: " }
                span { style: "color: #78350f;", "This example shows colored items with unique IDs. State management for add/remove requires callback patterns." }
            }

            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: 12,
                    row_height: 100.0,
                    margin: (10, 10),
                    compaction: CompactionType::FreePlacement,
                    render_item: |item: LayoutItem| {
                        let color = get_item_color(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: {color}; border-radius: 12px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: white; font-weight: 700; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                span { style: "font-size: 28px;", "{item.id}" }
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
        }
    }
}
