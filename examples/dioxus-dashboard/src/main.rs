use crate::examples::*;
use dioxus::prelude::*;
use sidebar::{Sidebar, SidebarItem};
use ui::HadroneDashboardStylesheet;

mod examples;
mod grid_helpers;
mod sidebar;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Debugger {},

    #[route("/basic")]
    Basic {},

    #[route("/no-dragging")]
    NoDragging {},

    #[route("/dynamic-add-remove")]
    DynamicAddRemove {},

    #[route("/gravity")]
    Gravity {},

    #[route("/aspect-ratio")]
    AspectRatio {},

    #[route("/min-max")]
    MinMax {},

    #[route("/collisions")]
    Collisions {},

    #[route("/responsive")]
    Responsive {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        HadroneDashboardStylesheet {}
        Router::<Route> {}
    }
}

#[component]
fn MainLayout() -> Element {
    let mut sidebar_open = use_signal(|| false);
    let route = use_route::<Route>();

    let route_str = match route {
        Route::Debugger {} => "/".to_string(),
        Route::Basic {} => "/basic".to_string(),
        Route::NoDragging {} => "/no-dragging".to_string(),
        Route::DynamicAddRemove {} => "/dynamic-add-remove".to_string(),
        Route::Gravity {} => "/gravity".to_string(),
        Route::AspectRatio {} => "/aspect-ratio".to_string(),
        Route::MinMax {} => "/min-max".to_string(),
        Route::Collisions {} => "/collisions".to_string(),
        Route::Responsive {} => "/responsive".to_string(),
    };

    let sidebar_items = vec![
        SidebarItem {
            icon: "⚡",
            label: "Grid Engine Debugger",
            route: "/",
            badge: Some("Main"),
        },
        SidebarItem {
            icon: "📦",
            label: "Basic Layout",
            route: "/basic",
            badge: None,
        },
        SidebarItem {
            icon: "🔒",
            label: "No Dragging",
            route: "/no-dragging",
            badge: None,
        },
        SidebarItem {
            icon: "➕",
            label: "Dynamic Items",
            route: "/dynamic-add-remove",
            badge: None,
        },
        SidebarItem {
            icon: "⬇️",
            label: "Gravity Compaction",
            route: "/gravity",
            badge: None,
        },
        SidebarItem {
            icon: "📐",
            label: "Aspect Ratio",
            route: "/aspect-ratio",
            badge: None,
        },
        SidebarItem {
            icon: "📏",
            label: "Min/Max Constraints",
            route: "/min-max",
            badge: None,
        },
        SidebarItem {
            icon: "💥",
            label: "Collision Handling",
            route: "/collisions",
            badge: None,
        },
        SidebarItem {
            icon: "📱",
            label: "Responsive",
            route: "/responsive",
            badge: None,
        },
    ];
    // Auto-close sidebar on route change for small screens
    {
        let mut sidebar_open = sidebar_open.clone();
        let _route = route.clone();
        use_effect(move || {
            // Close the sidebar when the route changes (helps mobile drawer UX).
            if sidebar_open() {
                sidebar_open.set(false);
            }
        });
    }

    rsx! {
        div { class: "layout-with-sidebar",
            Sidebar {
                items: sidebar_items,
                active_route: route_str,
                is_open: sidebar_open(),
                on_toggle: move |_| sidebar_open.toggle()
            }

            div { class: "main-content",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn Debugger() -> Element {
    rsx! { DebuggerExample {} }
}

#[component]
fn Basic() -> Element {
    rsx! { BasicExample {} }
}

#[component]
fn NoDragging() -> Element {
    rsx! { NoDraggingExample {} }
}

#[component]
fn DynamicAddRemove() -> Element {
    rsx! { DynamicAddRemoveExample {} }
}

#[component]
fn Gravity() -> Element {
    rsx! { GravityExample {} }
}

#[component]
fn AspectRatio() -> Element {
    rsx! { AspectRatioExample {} }
}

#[component]
fn MinMax() -> Element {
    rsx! { MinMaxExample {} }
}

#[component]
fn Collisions() -> Element {
    rsx! { CollisionsExample {} }
}

#[component]
fn Responsive() -> Element {
    rsx! { ResponsiveExample {} }
}
