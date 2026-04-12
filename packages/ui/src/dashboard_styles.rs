use dioxus::prelude::*;

const HADRONE_DASHBOARD_CSS: Asset = asset!("/assets/styling/hadrone-dashboard.css");

/// Injects shared styles for the web/desktop grid debugger demos.
#[component]
pub fn HadroneDashboardStylesheet() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: HADRONE_DASHBOARD_CSS }
    }
}
