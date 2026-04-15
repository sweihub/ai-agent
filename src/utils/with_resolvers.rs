#![allow(dead_code)]

pub fn with_resolvers<T>() -> (std::sync::mpsc::Receiver<T>, impl FnOnce(T), impl FnOnce(T)) {
    let (tx, rx) = std::sync::mpsc::channel();
    let resolve = move |v: T| {
        let _ = tx.send(v);
    };
    let reject = move |_: T| {};
    (rx, resolve, reject)
}
