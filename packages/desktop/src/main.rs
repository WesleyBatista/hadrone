use dioxus::prelude::*;

use ui::{HadroneDashboardStylesheet, Navbar};
use views::DashboardHome;

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopNavbar)]
    #[route("/")]
    DashboardHome {},
}

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        HadroneDashboardStylesheet {}
        Router::<Route> {}
    }
}

#[component]
fn DesktopNavbar() -> Element {
    rsx! {
        Navbar {
            Link {
                to: Route::DashboardHome {},
                "Home"
            }
        }

        Outlet::<Route> {}
    }
}
