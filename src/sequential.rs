// Source: /data/home/swei/claudecode/openclaudecode/src/utils/sequential.ts
use std::collections::VecDeque;

pub fn sequential<F, T, R>(
    f: F,
) -> impl Fn(T) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send>>
where
    F: Fn(T) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send>>
        + Send
        + Sync
        + 'static,
    T: Send + 'static,
    R: Send + 'static,
{
    let queue = std::sync::Mutex::new(VecDeque::new());
    let processing = std::sync::Mutex::new(false);

    move |args: T| {
        let queue = queue.clone();
        let processing = processing.clone();
        let f = f.clone();

        Box::pin(async move {
            let (tx, rx) = std::sync::mpsc::channel();

            {
                let mut q = queue.lock().unwrap();
                q.push_back((args, tx));
            }

            loop {
                let should_process = {
                    let mut p = processing.lock().unwrap();
                    if *p || q.lock().unwrap().is_empty() {
                        false
                    } else {
                        *p = true;
                        true
                    }
                };

                if !should_process {
                    break;
                }

                let item = {
                    let mut q = queue.lock().unwrap();
                    q.pop_front()
                };

                if let Some((args, tx)) = item {
                    let result = f(args).await;
                    let _ = tx.send(result);
                }

                {
                    let mut p = processing.lock().unwrap();
                    *p = false;
                }

                if queue.lock().unwrap().is_empty() {
                    break;
                }
            }

            rx.recv().unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sequential() {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0usize));
        let counter_clone = counter.clone();

        let wrapped = sequential(move |n: usize| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let mut c = counter.lock().unwrap();
                *c += n;
                *c
            })
        });

        let h1 = wrapped(1);
        let h2 = wrapped(2);
        let h3 = wrapped(3);

        let r1 = h1.await;
        let r2 = h2.await;
        let r3 = h3.await;

        let c = *counter.lock().unwrap();
        assert!(c >= 1);
    }
}
