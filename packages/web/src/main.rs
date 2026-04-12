use dioxus::prelude::*;

use ui::HadroneDashboardStylesheet;
use views::DashboardHome;

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebNavbar)]
    #[route("/")]
    DashboardHome {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        HadroneDashboardStylesheet {}

        Router::<Route> {}
    }
}

#[component]
fn WebNavbar() -> Element {
    rsx! {
        //     Navbar {
        //         Link {
        //             to: Route::Home {},
        //             "Home"
        //         }
        //     }
        Outlet::<Route> {}
    }
}
