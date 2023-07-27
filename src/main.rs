//use crate::{compose::Compose, sidebar::Sidebar};

use dioxus::prelude::*;

// https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API
// https://github.com/DioxusLabs/dioxus/pull/1080

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let css = include_str!(".styles.css");
    let v: Vec<u32> = (0..100).collect();
    let eval_provider = dioxus_html::prelude::use_eval(cx);
    println!("rendering app");

    use_future(cx, (), |()| {
        to_owned![eval_provider];
        async move {
            let eval = match eval_provider(
                r#"
                let el = document.getElementById("main");
                if el === null {
                    dioxus.send("could not find main");
                } else {
                    dioxus.send("found main!");
                }
                
                let msg = await dioxus.recv();
                console.log(msg);
            "#,
            ) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("use eval failed: {:?}", e);
                    return;
                }
            };
            match eval.recv().await {
                Ok(msg) => {
                    println!("got this from js: {msg}");
                }
                Err(e) => println!("eval failed: {e:?}"),
            };
        }
    });

    render! {
        style {
            "{css}"
        }
        main {
            id: "main",
            onscroll: move |_evt| {
                // doesn't fire
                println!("main scrolled");
            },
            div {
                class: "sidebar",
            },
            div {
                id: "compose",
                class: "compose",
                onscroll: move |_evt| {
                    // doesn't fire
                    println!("div scrolled");
                },
                ul {
                    id: "compose-list",
                    onscroll: move |_evt| {
                        // doesn't fire
                        println!("ul scrolled");
                    },
                    v.iter().map(|x| rsx!(li{"{x}"}))
                }
            },
        }
    }
}
