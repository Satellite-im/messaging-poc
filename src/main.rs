//use crate::{compose::Compose, sidebar::Sidebar};

use futures::StreamExt;
use std::collections::VecDeque;

use dioxus::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum JsMsg {
    // ex json: {"Add":1}
    Add(u32),
    Remove(u32),
    // ex json: {"Top":null}
    Top,
    Bottom,
}

// https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API
// https://github.com/DioxusLabs/dioxus/pull/1080

const OBSERVER_SCRIPT: &str = r###"
function observe_list() {
    var observer3 = new IntersectionObserver( (entries, observer) => {
        entries.forEach((entry) => {
            if (entry.isIntersecting) {
                dioxus.send("{\"Add\":" + entry.target.id + "}");
                if (!entry.target.nextElementSibling) {
                    dioxus.send("{\"Bottom\":null}");
                } else if (!entry.target.previousElementSibling) {
                    dioxus.send("{\"Top\":null}");
                    // todo: only disconnect in response to command...
                    observer.disconnect();
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
}


    observe_list();


"###;

fn main() {
    dioxus_desktop::launch(app);
}

#[inline_props]
fn render_msg_list(
    cx: Scope,
    msg_list: UseRef<SortedList<u32>>,
    to_render: Vec<u32>,
    to_take: UseState<usize>,
    conversation_len: usize,
) -> Element {
    println!("rendering list");

    let css = include_str!(".styles.css");
    let eval_provider = dioxus_html::prelude::use_eval(cx);
    let scroll_to: &UseRef<Option<u32>> = use_ref(cx, || None);

    let scroll_script = match scroll_to.read().as_ref() {
        Some(id) => {
            println!("scrolling to id {id}");
            let s = r##"
var message = document.getElementById("$MESSAGE_ID");
message.scrollIntoView({ behavior: 'instant', block: 'start' });
return "done";
"##;
            s.replace("$MESSAGE_ID", &format!("{id}"))
        }
        None => r#"window.scrollTo(0, document.body.scrollHeight); return "done";"#.into(),
    };

    let ch = use_coroutine(cx, |mut rx: UnboundedReceiver<()>| {
        to_owned![
            eval_provider,
            to_take,
            msg_list,
            scroll_to,
            conversation_len
        ];
        async move {
            println!("starting use_coroutine");
            while rx.next().await.is_some() {
                println!("use_coroutine loop");
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
                                            if y < conversation_len {
                                                let x = std::cmp::min(y + 20, conversation_len);
                                                *scroll_to.write() = msg_list.read().get_min();
                                                to_take.set(x);
                                                msg_list.write().clear();
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

    use_effect(cx, (&scroll_script), move |scroll_script| {
        to_owned![eval_provider, ch];
        async move {
            println!("use_effect");
            match eval_provider(&scroll_script) {
                Ok(eval) => {
                    if let Err(e) = eval.join().await {
                        eprintln!("failed to join eval: {:?}", e);
                    } else {
                        ch.send(());
                    }
                }
                Err(e) => {
                    eprintln!("eval failed: {:?}", e);
                }
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
                    onmounted: move |_| {
                        // only good for calling something the first time the element renders
                        println!("list is mounted");
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
        }
    }
}

fn app(cx: Scope) -> Element {
    let v: Vec<u32> = (0..100).collect();
    let to_take = use_state(cx, || 20);
    let _to_render: Vec<_> = v
        .iter()
        .rev()
        .take(*to_take.current())
        .rev()
        .cloned()
        .collect();
    println!("rendering app. to_take is {}", to_take.current());
    let msg_list = use_ref(cx, SortedList::new);

    render! {
        render_msg_list{
            msg_list: msg_list.clone(),
            to_render: _to_render,
            to_take: to_take.clone(),
            conversation_len: v.len(),
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

    fn clear(&mut self) {
        self.items.clear();
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
