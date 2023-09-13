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

    use_effect(cx, (), |_| {
        to_owned![eval_provider];
        async move {
            let eval = match eval_provider(
                r###"

                let options = {
                    root: null, //document.querySelector("#compose-list"),
                    rootMargin: "0px",
                    threshold: 0.75,
                };
                let observer = new IntersectionObserver( (entries, observer) => {
                     console.log(entries);
                    if (entries[0].isIntersecting) {
                        dioxus.send("intersection-top");
                   } else {
                       dioxus.send("removed intersection-top");
                   }
                }, options);

                observer.observe(document.querySelector("li:first-child"));

                let options2 = {
                    root: null, // document.querySelector("#compose-list"),
                    rootMargin: "0px",
                    threshold: 0.75,
                };
                let observer2 = new IntersectionObserver( (entries, observer) => {
                    // console.log(entries);
                    if (entries[0].isIntersecting) {
                         dioxus.send("intersection-bottom");
                    } else {
                        dioxus.send("removed intersection-bottom");
                    }
                   
                }, options);

                observer2.observe(document.querySelector("li:last-child"));

                let observer3 = new IntersectionObserver( (entries, observer) => {
                    entries.forEach((entry) => {
                        if (entry.isIntersecting) {
                            dioxus.send("new intersection: " + entry.target.id);
                        } else {
                            dioxus.send("removed intersection: " + entry.target.id);
                        }
                    });
                }, {
                    root: null,
                    rootMargin: "0px",
                    threshold: 0.75,
                });
                const elements = document.querySelectorAll("#compose-list > li");
                elements.forEach( (element) => {
                    let id = "#" + element.id;
                    // dioxus.send("observing " + id);
                    observer3.observe(element);
                });

                // const elem = document.getElementById("compose");
                // const rect = elem.getBoundingClientRect();
                // dioxus.send("rect height: " + rect["height"]);
                // dioxus.send("rect width: " + rect["width"]);
            "###,
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
                    v.iter().map(|x| rsx!(li {
                        id: "{x}",
                        "{x}"
                    }))
                }
            },
        }
    }
}
