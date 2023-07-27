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

    let script_var = r###"
    var g = function () {
        

        console.log("registered event handlers");
        var input = document.getElementById("compose-input");

        document.addEventListener("scroll", (event) => {
            if (window.scrollY === 0) {
                console.log("scrolled to top");
                input.value="top";
                input.onchange();
            }
            if (window.innerHeight + window.pageYOffset >= document.body.offsetHeight) {
                console.log("scrolled to bottom");
                input.value="bottom";
                input.onchange();
            }
        });

        // todo: why do none of these work? 
        var compose = document.getElementById("compose");
        var compose_list = document.getElementById("compose-list");
        var main = document.getElementById("main");
        
        compose.addEventListener('scroll', function(evt) {
            console.log("scrolled div");
            if (evt.target.scrollTop === 0) {
                console.log("scrolled to top");
            }
            if (evt.target.scrollTop === evt.target.scrollHeight) {
                console.log("scrolled to bottom");
            }
        }, false);

        compose_list.addEventListener('scroll', function(evt) {
            console.log("scrolled list");
            if (evt.target.scrollTop === 0) {
                console.log("scrolled to top");
            }
            if (evt.target.scrollTop === evt.target.scrollHeight) {
                console.log("scrolled to bottom");
            }
        }, false);

        main.addEventListener('scroll', function(evt) {
            console.log("scrolled main");
            if (evt.target.scrollTop === 0) {
                console.log("scrolled to top");
            }
            if (evt.target.scrollTop === evt.target.scrollHeight) {
                console.log("scrolled to bottom");
            }
        }, false);
    };
    g();
    "###;

    render! {
        style {
            "{css}"
        }
        main {
            id: "main",
            onscroll: move |evt| {
                println!("scrolling main");
            },
            div {
                class: "sidebar",
            },
            div {
                id: "compose",
                class: "compose",
                onscroll: move |evt| {
                    println!("scrolling compose");
                },
                input {
                    id: "compose-input",
                    class: "hidden-input",
                    placeholder: "test value",
                    onchange: move |evt| {
                        let value = &evt.value;
                        println!("{value}");
                    }
                },
                ul {
                    id: "compose-list",
                    onscroll: move |evt| {
                        println!("scrolling list");
                    },
                    v.iter().map(|x| rsx!(li{"{x}"}))
                }
            },
            script {
                "{script_var}"
            }
        }
    }
}
