use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use std::collections::HashSet;
use crate::grid_helpers::*;

pub const RSX_CODE: &str = r#"LayoutItem {
    id: "minW: 2".into(),
    x: 0,
    y: 0,
    w: 3,
    h: 2,
    min_w: Some(2),  // Key property!
    resize_handles: handles.clone(),
    ..Default::default()
}

LayoutItem {
    id: "maxW: 4".into(),
    max_w: Some(4),  // Width capped at 4 columns
    ..
}

LayoutItem {
    id: "minH: 3".into(),
    min_h: Some(3),  // Height minimum 3 rows
    ..
}"#;

fn get_minmax_color(id: &str) -> (&'static str, &'static str) {
    if id.contains("minW") {
        ("#ef4444", "min_w: 2")
    } else if id.contains("maxW") {
        ("#f97316", "max_w: 4")
    } else if id.contains("minH") {
        ("#22c55e", "min_h: 3")
    } else if id.contains("maxH") {
        ("#3b82f6", "max_h: 2")
    } else {
        ("#8b5cf6", "All constraints")
    }
}

fn default_layout() -> Vec<LayoutItem> {
    let mut handles = HashSet::new();
    handles.insert(ResizeHandle::SouthEast);
    
    vec![
        LayoutItem { id: "minW: 2".into(), x: 0, y: 0, w: 3, h: 2, min_w: Some(2), resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "maxW: 4".into(), x: 3, y: 0, w: 3, h: 2, max_w: Some(4), resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "minH: 3".into(), x: 6, y: 0, w: 3, h: 3, min_h: Some(3), resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "maxH: 2".into(), x: 9, y: 0, w: 3, h: 2, max_h: Some(2), resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "All Constraints".into(), x: 0, y: 2, w: 4, h: 3, min_w: Some(2), max_w: Some(6), min_h: Some(2), max_h: Some(4), resize_handles: handles.clone(), ..Default::default() },
    ]
}

#[component]
pub fn MinMaxExample() -> Element {
    let layout = use_signal(|| default_layout());
    
    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "Min/Max Size Constraints",
                description: "Items can have minimum and maximum width/height constraints. Try resizing each widget to see the limits.",
                show_code: true,
                code: Some(RSX_CODE),
                show_reset: false,
            }
            
            div { style: "margin-bottom: 24px; padding: 16px; background: #e0e7ff; border-radius: 12px; border: 1px solid #6366f1;",
                strong { style: "color: #4338ca;", "Size Constraints: " }
                span { style: "color: #3730a3;", "Each widget has different min/max constraints. Resize them to feel the limits." }
            }
            
            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: 12,
                    row_height: 80.0,
                    margin: (10, 10),
                    compaction: CompactionType::FreePlacement,
                    render_item: |item: LayoutItem| {
                        let (color, constraint_str) = get_minmax_color(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: {color}; border-radius: 8px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: white; font-weight: 700; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                span { style: "font-size: 14px;", "{item.id}" }
                                span { style: "font-size: 10px; opacity: 0.8;", "{constraint_str}" }
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
        }
    }
}
