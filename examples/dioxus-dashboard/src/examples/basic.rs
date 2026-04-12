use dioxus::prelude::*;
use hadrone_core::{CompactionType, LayoutItem};
use hadrone_dioxus::GridLayout;
use crate::examples::generate_random_layout;
use crate::grid_helpers::*;

const GRADIENT_COLORS: [(&str, &str); 6] = [
    ("#667eea", "#764ba2"),
    ("#f093fb", "#f5576c"),
    ("#4facfe", "#00f2fe"),
    ("#43e97b", "#38f9d7"),
    ("#fa709a", "#fee140"),
    ("#a8edea", "#fed6e3"),
];

fn get_gradient_color(id: &str) -> (String, String) {
    let idx = id.bytes().next().map(|b| b as usize % 6).unwrap_or(0);
    let (c1, c2) = GRADIENT_COLORS[idx];
    (c1.to_string(), c2.to_string())
}

fn default_layout() -> Vec<LayoutItem> {
    generate_random_layout(20, 12)
}

#[component]
pub fn BasicExample() -> Element {
    let mut layout = use_signal(|| default_layout());
    let mut cols = use_signal(|| 12);
    
    let on_reset = move |_| {
        layout.set(default_layout());
        cols.set(12);
    };
    
    rsx! {
        div { class: "example-content",
            ExampleHeader {
                title: "Basic Grid Layout",
                description: "A simple 12-column grid with 20 randomly positioned items. Drag and resize to see the layout respond.",
                show_code: true,
                code: Some(r#"GridLayout {
    layout,
    cols: 12,
    row_height: 60.0,
    margin: (10, 10),
    compaction: CompactionType::FreePlacement,
    render_item: |item: LayoutItem| { ... },
}"#),
                show_reset: true,
                on_reset: EventHandler::new(on_reset),
            }
            
            ExampleControls {
                ControlGroup { label: "Columns",
                    input {
                        r#type: "range",
                        min: "4",
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
                    button {
                        class: "example-controls__btn",
                        onclick: move |_| {
                            layout.set(generate_random_layout(20, cols()));
                        },
                        "Randomize Layout"
                    }
                }
            }
            
            div { style: "background: white; border-radius: 16px; padding: 20px; box-shadow: inset 0 2px 4px rgba(0,0,0,0.05);",
                GridLayout {
                    layout,
                    cols: cols(),
                    row_height: 60.0,
                    margin: (10, 10),
                    compaction: CompactionType::FreePlacement,
                    render_item: |item: LayoutItem| {
                        let (color1, color2) = get_gradient_color(&item.id);
                        rsx! {
                            div {
                                style: "width: 100%; height: 100%; background: linear-gradient(135deg, {color1} 0%, {color2} 100%); border-radius: 8px; display: flex; align-items: center; justify-content: center; color: white; font-weight: 700; font-size: 18px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                                "{item.id}"
                            }
                        }
                    },
                    on_layout_change: move |_| {}
                }
            }
        }
    }
}
