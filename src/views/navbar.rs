use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

/// Navigation bar with links to Dashboard and Settings
#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div {
            id: "navbar",
            Link {
                to: Route::Dashboard {},
                "🎬 Recordings"
            }
            Link {
                to: Route::Settings {},
                "⚙️ Settings"
            }
        }

        Outlet::<Route> {}
    }
}
