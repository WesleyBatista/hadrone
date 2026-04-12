use dioxus::prelude::*;
use dioxus_router::components::Link;

#[derive(Props, Clone, PartialEq)]
pub struct SidebarItem {
    pub icon: &'static str,
    pub label: &'static str,
    pub route: &'static str,
    pub badge: Option<&'static str>,
}

#[component]
pub fn Sidebar(
    items: Vec<SidebarItem>,
    active_route: String,
    is_open: bool,
    on_toggle: EventHandler<MouseEvent>,
) -> Element {
    let main_items: Vec<SidebarItem> = items.iter().filter(|i| i.route == "/").cloned().collect();
    let feature_items: Vec<SidebarItem> = items.iter().filter(|i| i.route != "/").cloned().collect();

    rsx! {
        button {
            class: "sidebar-toggle",
            onclick: move |e| on_toggle.call(e),
            aria_expanded: is_open,
            aria_controls: "sidebar",
            "☰"
        }
        
        if is_open {
            div {
                class: "sidebar-overlay sidebar-overlay--visible",
                onclick: move |e| on_toggle.call(e),
                role: "presentation"
            }
        }
        
        nav {
            id: "sidebar",
            role: "navigation",
            aria_hidden: !is_open,
            class: if is_open { "sidebar sidebar--open" } else { "sidebar" },
            
            div { class: "sidebar__header",
                div { class: "sidebar__title",
                    span { "⚡" }
                    span { "Hadrone" }
                }
                div { class: "sidebar__subtitle", "Grid Engine Debugger" }
            }
            
            div { class: "sidebar__nav",
                div { class: "sidebar__section", "Main Demo" }
                
                for item in main_items.iter() {
                    SidebarNavItem {
                        item: item.clone(),
                        is_active: active_route == item.route,
                        on_toggle: on_toggle.clone()
                    }
                }
                
                if !feature_items.is_empty() {
                    div { class: "sidebar__section", "Features" }
                    
                    for item in feature_items.iter() {
                        SidebarNavItem {
                            item: item.clone(),
                            is_active: active_route == item.route,
                            on_toggle: on_toggle.clone()
                        }
                    }
                }
            }
            
            div { class: "sidebar__footer",
                "Powered by "
                a { href: "https://github.com/anomalyco/hadrone", "Hadrone Core" }
            }
        }
    }
}

#[component]
fn SidebarNavItem(
    item: SidebarItem,
    is_active: bool,
    on_toggle: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        Link {
            to: item.route,
            class: if is_active { "sidebar__item sidebar__item--active" } else { "sidebar__item" },
            onclick: move |e| { on_toggle.call(e); },

            span { class: "sidebar__item-icon", "{item.icon}" }
            span { class: "sidebar__item-text", "{item.label}" }

            if let Some(badge) = item.badge {
                span { class: "sidebar__item-badge", "{badge}" }
            }
        }
    }
}
