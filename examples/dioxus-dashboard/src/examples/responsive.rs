use dioxus::prelude::*;
use hadrone_core::{LayoutItem, ResizeHandle};
use hadrone_dioxus::GridLayout;
use std::collections::HashSet;
use crate::grid_helpers::*;

pub const RSX_CODE: &str = r#"GridLayout {
    layout,
    cols: 12,  // Change this dynamically!
    row_height: 60.0,
    margin: (10, 10),
    compaction: CompactionType::FreePlacement,
    render_item: |item: LayoutItem| { /* ... */ },
}

// Common breakpoints:
// - Desktop: 12 cols
// - Tablet: 6 cols
// - Mobile: 4 cols"#;

fn get_responsive_color(id: &str) -> &'static str {
    match id {
        "Header" | "Footer" => "#1e293b",
        "Sidebar" => "#3b82f6",
        "Main" => "#22c55e",
        _ => "#f97316",
    }
}

fn default_layout() -> Vec<LayoutItem> {
    let mut handles = HashSet::new();
    handles.insert(ResizeHandle::SouthEast);
    
    vec![
        LayoutItem { id: "Header".into(), x: 0, y: 0, w: 12, h: 1, resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "Sidebar".into(), x: 0, y: 1, w: 3, h: 4, resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "Main".into(), x: 3, y: 1, w: 6, h: 4, resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "Aside".into(), x: 9, y: 1, w: 3, h: 4, resize_handles: handles.clone(), ..Default::default() },
        LayoutItem { id: "Footer".into(), x: 0, y: 5, w: 12, h: 1, resize_handles: handles.clone(), ..Default::default() },
    ]
}

#[component]
pub fn ResponsiveExample() -> Element {
    let mut cols = use_signal(|| 12);
    let layout = use_signal(|| default_layout());
    
    let current_cols = cols();
    
    let size_label = if current_cols >= 12 { 
        "Desktop (12 cols)" 
    } else if current_cols >= 8 { 
        "Laptop (10 cols)" 
    } else if current_cols >= 6 { 
        "Tablet (6 cols)" 
    } else { 
        "Mobile (4 cols)" 
    };
    
    let desktop_bg = if current_cols == 12 { "#3b82f6" } else { "#e2e8f0" };
    let desktop_color = if current_cols == 12 { "white" } else { "#64748b" };
    let tablet_bg = if current_cols == 6 { "#3b82f6" } else { "#e2e8f0" };
    let tablet_color = if current_cols == 6 { "white" } else { "#64748b" };
    let mobile_bg = if current_cols == 4 { "#3b82f6" } else { "#e2e8f0" };
    let mobile_color = if current_cols == 4 { "white" } else { "#64748b" };
    
    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "Responsive Breakpoints",
                description: "Change the number of columns to simulate different screen sizes. A 12-column layout becomes a 6-column tablet layout or 4-column mobile layout.",
                show_code: true,
                code: Some(RSX_CODE),
                show_reset: false,
            }
            
            div { class: "example-controls",
                div { class: "example-controls__group",
                    label { class: "example-controls__label", "Columns (Screen Size)" }
                    input {
                        r#type: "range",
                        min: "4",
                        max: "12",
                        value: "{current_cols}",
                        style: "width: 150px; cursor: pointer;",
                        oninput: move |e| {
                            let val = e.value().parse::<i32>().unwrap_or(12);
                            cols.set(val);
                        }
                    }
                    span { style: "font-size: 12px; color: #64748b;", "{size_label}" }
                }
            }
            
            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: current_cols,
                    row_height: 60.0,
                    margin: (10, 10),
                    compaction: hadrone_core::CompactionType::FreePlacement,
                    render_item: |item: LayoutItem| {
                        let color = get_responsive_color(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: {color}; border-radius: 8px; display: flex; align-items: center; justify-content: center; color: white; font-weight: 700; font-size: 14px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                "{item.id}"
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
            
            div { style: "margin-top: 16px; display: flex; gap: 8px;",
                button {
                    style: "padding: 8px 16px; background: {desktop_bg}; color: {desktop_color}; border: 1px solid #e2e8f0; border-radius: 8px; cursor: pointer; font-size: 13px; font-weight: 600;",
                    onclick: move |_| cols.set(12),
                    "Desktop"
                }
                button {
                    style: "padding: 8px 16px; background: {tablet_bg}; color: {tablet_color}; border: 1px solid #e2e8f0; border-radius: 8px; cursor: pointer; font-size: 13px; font-weight: 600;",
                    onclick: move |_| cols.set(6),
                    "Tablet"
                }
                button {
                    style: "padding: 8px 16px; background: {mobile_bg}; color: {mobile_color}; border: 1px solid #e2e8f0; border-radius: 8px; cursor: pointer; font-size: 13px; font-weight: 600;",
                    onclick: move |_| cols.set(4),
                    "Mobile"
                }
            }
        }
    }
}
