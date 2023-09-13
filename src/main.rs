//use crate::{compose::Compose, sidebar::Sidebar};

use std::collections::VecDeque;

use dioxus::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum JsMsg {
    // ex json: {"Add":1}
    Add(i32),
    Remove(i32),
    // ex json: {"Top":null}
    Top,
    Bottom,
}

// https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API
// https://github.com/DioxusLabs/dioxus/pull/1080

const OBSERVER_SCRIPT: &str = r###"

let options = {
    root: null, //document.querySelector("#compose-list"),
    rootMargin: "0px",
    threshold: 0.75,
};
let observer = new IntersectionObserver( (entries, observer) => {
    if (entries[0].isIntersecting) {
        dioxus.send("{\"Top\":null}");
   }
}, options);

observer.observe(document.querySelector("li:first-child"));

let options2 = {
    root: null, // document.querySelector("#compose-list"),
    rootMargin: "0px",
    threshold: 0.75,
};
let observer2 = new IntersectionObserver( (entries, observer) => {
    if (entries[0].isIntersecting) {
         dioxus.send("{\"Bottom\":null}");
    }
}, options);

observer2.observe(document.querySelector("li:last-child"));

let observer3 = new IntersectionObserver( (entries, observer) => {
    entries.forEach((entry) => {
        if (entry.isIntersecting) {
            dioxus.send("{\"Add\":" + entry.target.id + "}");
        } else {
            dioxus.send("{\"Remove\":" + entry.target.id + "}");
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

window.scrollTo(0, document.body.scrollHeight);

// const elem = document.getElementById("compose");
// const rect = elem.getBoundingClientRect();
// dioxus.send("rect height: " + rect["height"]);
// dioxus.send("rect width: " + rect["width"]);
"###;

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let css = include_str!(".styles.css");
    let v: Vec<u32> = (0..100).collect();
    let eval_provider = dioxus_html::prelude::use_eval(cx);
    println!("rendering app");

    let to_take = use_state(cx, || 20);
    let should_scroll_to_top = use_ref(cx, || false);
    let to_render: Vec<_> = v
        .iter()
        .rev()
        .take(*to_take.current())
        .rev()
        .cloned()
        .collect();

    let msg_list = use_ref(cx, SortedList::new);

    if *should_scroll_to_top.read() {
        *should_scroll_to_top.write_silent() = false;
        if let Some(id) = msg_list.read().get_min() {
            println!("scrolling to top: {}", id);
            let scroll_script = format!("const elements = document.querySelectorAll(\"#{id}\"); elements.forEach((elem) => elem.scrollIntoView(true));");
            _ = eval_provider(&scroll_script);
        }
    }

    use_future(cx, (), |_| {
        to_owned![eval_provider, to_take, msg_list, should_scroll_to_top];
        async move {
            let eval = match eval_provider(OBSERVER_SCRIPT) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("use eval failed: {:?}", e);
                    return;
                }
            };

            loop {
                match eval.recv().await {
                    Ok(msg) => {
                        //println!("got this from js: {msg}");
                        if let Some(s) = msg.as_str() {
                            match serde_json::from_str::<JsMsg>(s) {
                                Ok(msg) => match msg {
                                    JsMsg::Add(x) => {
                                        msg_list.write_silent().insert(x);
                                        println!(
                                            "new max: {:?}; new min: {:?}",
                                            msg_list.read().get_max(),
                                            msg_list.read().get_min()
                                        );
                                    }
                                    JsMsg::Remove(x) => {
                                        msg_list.write_silent().remove(x);
                                        println!(
                                            "new max: {:?}; new min: {:?}",
                                            msg_list.read().get_max(),
                                            msg_list.read().get_min()
                                        );
                                    }
                                    JsMsg::Top => {
                                        println!("top reached");
                                        let y = *to_take.current();
                                        if y < v.len() {
                                            let x = std::cmp::min(y + 20, v.len());
                                            *should_scroll_to_top.write_silent() = true;
                                            to_take.set(x);
                                        }
                                    }
                                    JsMsg::Bottom => {
                                        println!("bottom reached");
                                    }
                                },
                                Err(e) => {
                                    eprintln!("failed to deserialize message: {}: {}", s, e);
                                }
                            }
                        }
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
                    to_render.iter().map(|x| rsx!(li {
                        id: "{x}",
                        "{x}"
                    }))
                }
            },
        }
    }
}

struct SortedList<T>
where
    T: Ord,
{
    items: VecDeque<T>,
}

impl<T> SortedList<T>
where
    T: Ord + Clone + std::fmt::Debug,
{
    fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    fn insert(&mut self, val: T) {
        if self.items.is_empty() {
            self.items.push_back(val);
        } else if self.items.front().map(|x| x >= &val).unwrap_or(false) {
            self.items.push_front(val);
        } else if self.items.back().map(|x| x <= &val).unwrap_or(false) {
            self.items.push_back(val);
        } else {
            println!("invalid insert: {:?}", val);
        }
    }

    fn remove(&mut self, val: T) {
        if self.items.front().map(|x| x == &val).unwrap_or(false) {
            self.items.pop_front();
        } else if self.items.back().map(|x| x == &val).unwrap_or(false) {
            self.items.pop_back();
        } else {
            // println!("invalid remove: {:?}", val);
        }
    }

    fn get_min(&self) -> Option<T> {
        self.items.front().cloned()
    }

    fn get_max(&self) -> Option<T> {
        self.items.back().cloned()
    }

    fn get_idx(&self, idx: usize) -> Option<T> {
        self.items.get(idx).cloned()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sorted_list_test1() {
        let mut l = SortedList::new();
        l.insert(1);
        l.insert(2);

        assert_eq!(1, l.get_idx(0).unwrap());
        assert_eq!(2, l.get_idx(1).unwrap());

        l.insert(0);
        assert_eq!(0, l.get_idx(0).unwrap());
        assert_eq!(2, l.get_idx(2).unwrap());
    }

    #[test]
    fn sorted_list_test2() {
        let mut l = SortedList::new();
        l.insert(1);
        l.insert(2);
        l.insert(3);
        l.insert(4);

        l.remove(1);
        assert_eq!(2, l.get_idx(0).unwrap());

        l.remove(4);
        assert_eq!(3, l.get_idx(1).unwrap());
    }
}
