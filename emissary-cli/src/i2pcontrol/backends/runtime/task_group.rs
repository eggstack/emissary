use std::{future::Future, sync::Arc, time::Duration};

use tokio::{sync::Semaphore, task::JoinSet};

/// Bounded task ownership for one runtime instance.
pub(super) struct BoundedTaskGroup {
    tasks: JoinSet<()>,
    permits: Arc<Semaphore>,
}

impl BoundedTaskGroup {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            tasks: JoinSet::new(),
            permits: Arc::new(Semaphore::new(limit)),
        }
    }

    pub(super) fn try_spawn<F>(&mut self, task: F) -> bool
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let Ok(permit) = Arc::clone(&self.permits).try_acquire_owned() else {
            return false;
        };

        self.tasks.spawn(async move {
            let _permit = permit;
            task.await;
        });
        true
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub(super) async fn join_next(&mut self) -> Option<Result<(), tokio::task::JoinError>> {
        self.tasks.join_next().await
    }

    pub(super) async fn drain(&mut self, timeout: Duration) {
        if self.tasks.is_empty() {
            return;
        }

        let drain = async { while self.tasks.join_next().await.is_some() {} };

        if tokio::time::timeout(timeout, drain).await.is_err() {
            self.tasks.abort_all();
            while self.tasks.join_next().await.is_some() {}
        }
    }
}
