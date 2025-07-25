use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SchedulerService<S, ST> {
    inner: Arc<Mutex<S>>,
    strategy: ST,
}

impl<S, ST> SchedulerService<S, ST> {
    fn new(inner: S, strategy: ST) -> Self {
        SchedulerService {
            inner: Arc::new(Mutex::new(inner)),
            strategy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service() {
        let svc = SchedulerService::new((), ());
    }
}
