//use crate::{compose::Compose, sidebar::Sidebar};

use std::{cmp::Ordering, collections::VecDeque};

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
        match self.items.front() {
            None => self.items.push_back(val),
            Some(x) => match x.cmp(&val) {
                Ordering::Greater | Ordering::Equal => self.items.push_front(val),
                Ordering::Less => self.items.push_back(val),
            },
        }
    }

    fn remove(&mut self, val: T) {
        let item_removed = match self.items.front() {
            None => return,
            Some(x) => match x.cmp(&val) {
                Ordering::Equal => self.items.pop_front(),
                _ => self.items.pop_back(),
            },
        };
        assert_eq!(item_removed, Some(val));
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
