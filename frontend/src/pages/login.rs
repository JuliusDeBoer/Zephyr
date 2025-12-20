use std::collections::HashMap;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Route;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginData {
    email: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    rsx!(
        div {
            class: "w-full min-h-screen flex justify-center items-center flex-col",
            h1 {
                class: "text-6xl font-semibold text-red-800 mb-8",
                "Zephyr"
            },
            form {
                class: "flex justify-center items-center flex-col",
                onsubmit: |event: FormEvent| async move {
                    event.prevent_default();

                    let map: HashMap<_, _> = event.values().into_iter().collect();
                    let data = LoginData {
                        email: match &map["email"] {
                            FormValue::Text(v) => v.clone(),
                            // TODO(Julius): Please fix this
                            FormValue::File(_) => panic!("How on earth did a File get in there?"),
                        },
                        password: match &map["password"] {
                            FormValue::Text(v) => v.clone(),
                            // TODO(Julius): Please fix this
                            FormValue::File(_) => panic!("How on earth did a File get in there?"),
                        }
                    };

                    reqwest::Client::new()
                        .post("http://localhost:3000/auth/login")
                        // .body(serde_json::to_string(&data).expect("Could not serialize body"))
                        .json(&data)
                        .send()
                        .await
                        .expect("Could not login");
                },

                input {
                    placeholder: "Email",
                    class: "border-black border p-3 rounded mb-3",
                    required: true,
                    name: "email",
                    type: "email",
                },
                input {
                    placeholder: "Password",
                    class: "border-black border p-3 rounded mb-3",
                    required: true,
                    name: "password",
                    type: "password",
                },
                input {
                    type: "submit",
                    class: "bg-green-400 px-8 py-2 rounded text-lg text-white font-bold hover:bg-green-800 cursor-pointer",
                    value: "Inloggen"
                }
            }
            Link { to: Route::SignUp {}, "Sign up instead"}
        }
    )
}
