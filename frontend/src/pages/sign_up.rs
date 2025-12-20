use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn SignUp() -> Element {
    rsx!(
        div {
            class: "w-full min-h-screen flex justify-center items-center flex-col",
            h1 {
                class: "text-5xl",
                "Sign-up page"
            },
            p {
                "Frontend is my passion"
            }
            Link { to: Route::Login {}, "Log in instead"}
        }
    )
}
