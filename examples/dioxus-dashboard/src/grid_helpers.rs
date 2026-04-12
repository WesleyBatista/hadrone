use dioxus::prelude::*;
use hadrone_core::LayoutItem;

pub fn default_widget(item: &LayoutItem) -> Element {
    let aspect_badge = item.aspect_ratio.map(|ar| {
        rsx! {
            span {
                style: "font-size: 10px; background: #e0f2fe; color: #0369a1; padding: 2px 6px; border-radius: 4px; font-weight: 700;",
                "LOCK {ar:.1}"
            }
        }
    });
    
    rsx! {
        div {
            style: "width: 100%; height: 100%; background: white; border: 1px solid #e2e8f0; border-radius: 12px; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);",
            
            div {
                style: "height: 40px; background: #f8fafc; border-bottom: 1px solid #e2e8f0; display: flex; align-items: center; justify-content: space-between; padding: 0 16px;",
                span { 
                    style: "font-size: 13px; font-weight: 800; color: #1e293b;", 
                    "{item.id.to_uppercase()}" 
                }
                {aspect_badge}
            }
            
            div {
                style: "flex: 1; padding: 20px; display: flex; flex-direction: column; gap: 12px;",
                
                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-family: ui-monospace, monospace; font-size: 11px; color: #64748b;",
                    div { "OFFSET: ({item.x}, {item.y})" }
                    div { "SIZE: [{item.w} x {item.h}]" }
                }
                
                div {
                    style: "flex: 1; min-height: 0; background: #f1f5f9; border: 2px dashed #cbd5e1; border-radius: 8px; display: flex; align-items: center; justify-content: center; color: #94a3b8; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em;",
                    "Widget Content"
                }
            }
        }
    }
}

#[component]
pub fn ExampleHeader(
    title: &'static str, 
    description: &'static str,
    #[props(default = false)] show_code: bool,
    #[props(default = None)] code: Option<&'static str>,
    #[props(default = false)] show_reset: bool,
    on_reset: Option<EventHandler<MouseEvent>>,
) -> Element {
    let mut show_snippet = use_signal(|| false);
    
    rsx! {
        div { class: "example-header",
            h1 { class: "example-header__title", "{title}" }
            p { class: "example-header__desc", "{description}" }
            
            if show_reset || show_code {
                div { class: "example-header__actions",
                    if show_reset {
                        button {
                            class: "example-header__action-btn",
                            onclick: move |e| {
                                if let Some(handler) = on_reset {
                                    handler.call(e);
                                }
                            },
                            "↺ Reset Layout"
                        }
                    }
                    
                    if show_code && code.is_some() {
                        button {
                            class: if show_snippet() { "example-header__action-btn example-header__action-btn--active" } else { "example-header__action-btn" },
                            onclick: move |_| show_snippet.toggle(),
                            "{{}} Code"
                        }
                    }
                }
            }
            
            if show_code && code.is_some() && show_snippet() {
                CodeSnippet { code: code.unwrap() }
            }
        }
    }
}

#[component]
pub fn CodeSnippet(code: &'static str) -> Element {
    let highlighted = highlight_rust_code(code);
    
    rsx! {
        div { class: "code-snippet",
            pre { class: "code-snippet__pre",
                span { dangerous_inner_html: highlighted }
            }
        }
    }
}

pub fn highlight_rust_code(code: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];
        
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            result.push_str("<span class=\"code-comment\">");
            while i < chars.len() && chars[i] != '\n' {
                result.push(chars[i]);
                i += 1;
            }
            result.push_str("</span>");
        } else if c == '#' {
            let mut attr = String::from("#");
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                attr.push(chars[i]);
                i += 1;
            }
            result.push_str(&format!("<span class=\"code-macro\">{}</span>", attr));
        } else if c == '"' {
            result.push_str("<span class=\"code-string\">\"");
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    result.push(chars[i]);
                    i += 1;
                    result.push(chars[i]);
                    i += 1;
                } else if chars[i] == '"' {
                    result.push_str("\"</span>");
                    i += 1;
                    break;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
        } else if c.is_alphabetic() || c == '_' {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]);
                i += 1;
            }
            
            let classified = match word.as_str() {
                "use" | "pub" | "fn" | "let" | "mut" | "move" | "if" | "else" | "for" | "in" | "match" | "Some" | "None" | "true" | "false" => {
                    format!("<span class=\"code-keyword\">{}</span>", word)
                }
                "Signal" | "Element" | "Component" | "EventHandler" | "MouseEvent" | "use_signal" | "use_memo" | "use_effect" | "rsx" => {
                    format!("<span class=\"code-function\">{}</span>", word)
                }
                "String" | "i32" | "i64" | "f32" | "f64" | "usize" | "bool" | "Vec" | "HashSet" | "Option" | "Result" | "CollisionStrategy" | "CompactionType" | "LayoutItem" | "GridLayout" | "ResizeHandle" => {
                    format!("<span class=\"code-type\">{}</span>", word)
                }
                _ => word,
            };
            result.push_str(&classified);
        } else if c == ':' && i + 1 < chars.len() && chars[i + 1] == ':' {
            result.push_str("<span class=\"code-type\">::</span>");
            i += 2;
        } else {
            result.push(c);
            i += 1;
        }
    }
    
    result
}

#[component]
pub fn ExampleControls(children: Element) -> Element {
    rsx! {
        div { class: "example-controls",
            {children}
        }
    }
}

#[component]
pub fn ControlGroup(label: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "example-controls__group",
            label { class: "example-controls__label", "{label}" }
            {children}
        }
    }
}

#[component]
pub fn ControlButton(label: &'static str, onclick: EventHandler<MouseEvent>, secondary: bool) -> Element {
    let class = if secondary { "example-controls__btn example-controls__btn--secondary" } else { "example-controls__btn" };
    rsx! {
        button {
            class: "{class}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}
