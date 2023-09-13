//use crate::{compose::Compose, sidebar::Sidebar};

use futures::StreamExt;
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
let observer3 = new IntersectionObserver( (entries, observer) => {
    entries.forEach((entry) => {
        if (entry.isIntersecting) {
            dioxus.send("{\"Add\":" + entry.target.id + "}");
            if (!entry.target.nextElementSibling) {
                dioxus.send("{\"Bottom\":null}");
            } else if (!entry.target.previousElementSibling) {
                dioxus.send("{\"Top\":null}");
                //observer.disconnect();
            }
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
    observer3.observe(element);
});
"###;

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let css = include_str!(".styles.css");
    let v: Vec<u32> = (0..100).collect();
    let eval_provider = dioxus_html::prelude::use_eval(cx);

    let to_take = use_state(cx, || 20);
    let scroll_to: &UseRef<Option<u32>> = use_ref(cx, || None);
    let effect_id = use_state(cx, || 0);

    let to_render: Vec<_> = v
        .iter()
        .rev()
        .take(*to_take.current())
        .rev()
        .cloned()
        .collect();
    println!("rendering app. to_take is {}", to_take.current());
    let msg_list = use_ref(cx, SortedList::new);

    let scroll_script = match scroll_to.read().as_ref() {
        Some(id) => {
            println!("scrolling to id {id}");
            let s = r##"
var message = document.getElementById("$MESSAGE_ID");
message.scrollIntoView({ behavior: 'smooth', block: 'start' });
"##;
            s.replace("$MESSAGE_ID", &format!("{id}"))
        }
        None => "window.scrollTo(0, document.body.scrollHeight);".into(),
    };

    if scroll_to.read().is_some() {
        scroll_to.write_silent().take();
    }

    let ch = use_coroutine(cx, move |mut rx: UnboundedReceiver<()>| {
        to_owned![eval_provider, to_take, msg_list, scroll_to, v, effect_id];
        async move {
            println!("starting use_future");
            while rx.next().await.is_some() {
                println!("use_future loop");
                let eval = match eval_provider(OBSERVER_SCRIPT) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("use eval failed: {:?}", e);
                        return;
                    }
                };

                'HANDLE_EVAL: loop {
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
                                                *scroll_to.write() =
                                                    msg_list.read().get_min().map(|x| x as _);
                                                to_take.set(x);
                                                effect_id.with_mut(|x| *x = (*x + 1) % 3);
                                                //let _ = eval.join().await;
                                                break 'HANDLE_EVAL;
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
        }
    });

    use_effect(cx, (&*effect_id.get()), move |_| {
        to_owned![ch];
        async move {
            println!("use_effect");
            ch.send(());
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
                    onmounted: move |_| {
                        // todo: handle scrolling here
                        // only good for calling something the first time the element renders
                        println!("list is mounted");
                        ch.send(());
                    },
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
            script { scroll_script },
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
