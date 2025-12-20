use crate::pages::login::Login;
use crate::pages::sign_up::SignUp;
use dioxus::prelude::*;

mod pages;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/auth/login")]
    Login {},
    #[route("/auth/sign-up")]
    SignUp {},
}

#[allow(clippy::volatile_composites)]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[allow(clippy::volatile_composites)]
const MAIN_CSS: Asset = asset!("/assets/main.css");
#[allow(clippy::volatile_composites)]
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Home page
#[component]
fn Home() -> Element {
    let nav = navigator();
    nav.push(Route::Login {});
    rsx! {}
}
