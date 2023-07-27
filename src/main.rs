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

    use_future(cx, (), |_| {
        to_owned![eval_provider];
        async move {
            let eval = match eval_provider(
                r#"
                function handle_scroll(event) {
                    if (window.scrollY === 0) {
                        console.log("scrolled to top");
                        dioxus.send("top");
                    }
                    if (window.innerHeight + window.pageYOffset >= document.body.offsetHeight) {
                        console.log("scrolled to bottom");
                        dioxus.send("bottom");
                    }
                }
                if (document.onscroll === null) {
                    document.addEventListener("scroll", handle_scroll);
                }
            "#,
            ) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("use eval failed: {:?}", e);
                    return;
                }
            };
            loop {
                match eval.recv().await {
                    Ok(msg) => {
                        println!("got this from js: {msg}");
                    }
                    Err(e) => {
                        println!("eval failed: {e:?}");
                        break;
                    }
                };
            }
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
