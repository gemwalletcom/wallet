use tokio::sync::watch;

#[derive(Clone)]
pub struct Connections(watch::Sender<usize>);

struct Connection(watch::Sender<usize>);

impl Drop for Connection {
    fn drop(&mut self) {
        self.0.send_modify(|count| *count -= 1);
    }
}

impl Default for Connections {
    fn default() -> Self {
        Self(watch::Sender::new(0))
    }
}

impl Connections {
    pub fn track<F: Future>(&self, future: F) -> impl Future<Output = F::Output> + use<F> {
        self.0.send_modify(|count| *count += 1);
        let connection = Connection(self.0.clone());
        async move {
            let _connection = connection;
            future.await
        }
    }

    pub async fn wait(&self) {
        let mut count = self.0.subscribe();
        while *count.borrow_and_update() > 0 && count.changed().await.is_ok() {}
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::oneshot;
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn test_track() {
        let connections = Connections::default();

        let connection = connections.track(async {});
        assert_eq!(*connections.0.borrow(), 1);
        connection.await;
        assert_eq!(*connections.0.borrow(), 0);

        drop(connections.track(async {}));
        assert_eq!(*connections.0.borrow(), 0);
    }

    #[tokio::test]
    async fn test_wait() {
        let connections = Connections::default();
        connections.wait().await;

        let (release, released) = oneshot::channel::<()>();
        let connection = tokio::spawn(connections.track(async move { released.await.unwrap() }));
        assert!(timeout(Duration::from_millis(20), connections.wait()).await.is_err());

        release.send(()).unwrap();
        connection.await.unwrap();
        connections.wait().await;
        assert_eq!(*connections.0.borrow(), 0);
    }
}
