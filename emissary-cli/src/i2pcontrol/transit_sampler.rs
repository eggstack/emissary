//! Request-independent RouterInfo transit bandwidth sampling.
//!
//! This sampler is deliberately owned by the optional I2PControl adapter. It
//! reads the existing cumulative transit counter but does not add accounting,
//! timers, or lifecycle work to the router core traffic path.

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    time::Duration,
};

use tokio::{
    task::JoinHandle,
    time::{self, Instant, MissedTickBehavior},
};

use super::production::EventMetrics;

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const TRANSIT_WINDOW: Duration = Duration::from_secs(15);
const STALE_AFTER: Duration = Duration::from_secs(3);
const MAX_SAMPLES: usize = 17;

const NO_VALUE_YET: &str = "transit 15-second sampler warming up";
const SOURCE_UNAVAILABLE: &str = "authoritative cumulative transit source unavailable";
const STALE_VALUE: &str = "transit 15-second sampler is stale";

#[derive(Debug, Default)]
struct TransitWindow {
    samples: VecDeque<(Instant, u64)>,
    value: Option<(Instant, u64)>,
    source_available: bool,
}

impl TransitWindow {
    fn observe(&mut self, now: Instant, bytes: Option<u64>) {
        let Some(bytes) = bytes else {
            self.samples.clear();
            self.value = None;
            self.source_available = false;
            return;
        };
        self.source_available = true;

        if self.samples.back().is_some_and(|(_, previous)| bytes < *previous) {
            self.samples.clear();
            self.value = None;
        }

        if self.samples.back().is_some_and(|(timestamp, _)| *timestamp == now) {
            *self.samples.back_mut().expect("sample exists") = (now, bytes);
        } else {
            self.samples.push_back((now, bytes));
        }

        while self
            .samples
            .front()
            .is_some_and(|(timestamp, _)| now.duration_since(*timestamp) > TRANSIT_WINDOW)
        {
            self.samples.pop_front();
        }
        while self.samples.len() > MAX_SAMPLES {
            self.samples.pop_front();
        }

        let Some((oldest, oldest_bytes)) = self.samples.front().copied() else {
            self.value = None;
            return;
        };
        let elapsed = now.duration_since(oldest);
        let Some(delta_bytes) = bytes.checked_sub(oldest_bytes) else {
            self.value = None;
            return;
        };
        if elapsed < TRANSIT_WINDOW {
            self.value = None;
            return;
        }

        let elapsed_millis = elapsed.as_millis();
        let rate = (u128::from(delta_bytes) * 1_000)
            .checked_div(elapsed_millis)
            .unwrap_or(0)
            .min(u128::from(u64::MAX)) as u64;
        self.value = Some((now, rate));
    }

    fn value(&self, now: Instant) -> Result<u64, &'static str> {
        if !self.source_available {
            return Err(SOURCE_UNAVAILABLE);
        }
        let Some((updated, value)) = self.value else {
            return Err(NO_VALUE_YET);
        };
        if now.duration_since(updated) > STALE_AFTER {
            return Err(STALE_VALUE);
        }
        Ok(value)
    }
}

/// One bounded I2PControl-owned transit sampler.
pub(crate) struct TransitBandwidthSampler {
    state: Arc<RwLock<TransitWindow>>,
    cancellation: tokio::sync::watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl TransitBandwidthSampler {
    pub(crate) fn start(metrics: Arc<dyn EventMetrics>) -> Option<Arc<Self>> {
        metrics.transit_bytes_snapshot()?;
        tokio::runtime::Handle::try_current().ok()?;

        let state = Arc::new(RwLock::new(TransitWindow {
            source_available: true,
            ..TransitWindow::default()
        }));
        let (cancellation, mut cancelled) = tokio::sync::watch::channel(false);
        let task_state = Arc::clone(&state);
        let task_metrics = Arc::clone(&metrics);
        let task = tokio::spawn(async move {
            let first_tick = Instant::now() + SAMPLE_INTERVAL;
            let mut interval = time::interval_at(first_tick, SAMPLE_INTERVAL);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    result = cancelled.changed() => {
                        if result.is_err() || *cancelled.borrow() {
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        let now = Instant::now();
                        let bytes = task_metrics.transit_bytes_snapshot();
                        if let Ok(mut state) = task_state.write() {
                            state.observe(now, bytes);
                        }
                    }
                }
            }
        });

        Some(Arc::new(Self {
            state,
            cancellation,
            task,
        }))
    }

    pub(crate) fn snapshot(&self) -> Result<u64, &'static str> {
        self.state.read().map_err(|_| SOURCE_UNAVAILABLE)?.value(Instant::now())
    }

    #[cfg(test)]
    fn stop(&self) {
        let _ = self.cancellation.send(true);
    }

    #[cfg(test)]
    fn is_finished(&self) -> bool {
        self.task.is_finished()
    }
}

impl Drop for TransitBandwidthSampler {
    fn drop(&mut self) {
        let _ = self.cancellation.send(true);
        self.task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_floor_bytes_per_second_from_actual_elapsed_time() {
        let start = Instant::now();
        let mut window = TransitWindow::default();
        window.observe(start, Some(0));
        window.observe(start + Duration::from_millis(7_500), Some(7_500));
        window.observe(start + Duration::from_millis(15_000), Some(30_000));
        assert_eq!(window.value(start + TRANSIT_WINDOW), Ok(2_000));

        window.observe(start + Duration::from_millis(30_500), Some(30_000));
        assert_eq!(
            window.value(start + Duration::from_millis(30_500)),
            Err(NO_VALUE_YET)
        );
    }

    #[test]
    fn warmup_requires_a_complete_window_and_zero_traffic_is_real_zero() {
        let start = Instant::now();
        let mut window = TransitWindow::default();
        window.observe(start, Some(100));
        assert_eq!(window.value(start), Err(NO_VALUE_YET));
        window.observe(start + TRANSIT_WINDOW, Some(100));
        assert_eq!(window.value(start + TRANSIT_WINDOW), Ok(0));
    }

    #[test]
    fn counter_reset_starts_a_new_generation() {
        let start = Instant::now();
        let mut window = TransitWindow::default();
        window.observe(start, Some(100));
        window.observe(start + TRANSIT_WINDOW, Some(10_000));
        assert_eq!(window.value(start + TRANSIT_WINDOW), Ok(660));

        window.observe(start + TRANSIT_WINDOW + SAMPLE_INTERVAL, Some(10));
        assert_eq!(
            window.value(start + TRANSIT_WINDOW + SAMPLE_INTERVAL),
            Err(NO_VALUE_YET)
        );
    }

    #[test]
    fn source_failure_clears_current_value() {
        let start = Instant::now();
        let mut window = TransitWindow::default();
        window.observe(start, Some(0));
        window.observe(start + TRANSIT_WINDOW, Some(15_000));
        assert_eq!(window.value(start + TRANSIT_WINDOW), Ok(1_000));

        window.observe(start + TRANSIT_WINDOW + SAMPLE_INTERVAL, None);
        assert_eq!(
            window.value(start + TRANSIT_WINDOW + SAMPLE_INTERVAL),
            Err(SOURCE_UNAVAILABLE)
        );
    }

    #[test]
    fn stale_value_is_not_presented_as_current() {
        let start = Instant::now();
        let mut window = TransitWindow::default();
        window.observe(start, Some(0));
        window.observe(start + TRANSIT_WINDOW, Some(15_000));
        assert_eq!(
            window.value(start + TRANSIT_WINDOW + STALE_AFTER),
            Ok(1_000)
        );
        assert_eq!(
            window.value(start + TRANSIT_WINDOW + STALE_AFTER + SAMPLE_INTERVAL),
            Err(STALE_VALUE)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn sampler_runs_independently_of_router_info_reads_and_stops() {
        let metrics = Arc::new(TestMetrics::default());
        let sampler = TransitBandwidthSampler::start(metrics.clone()).expect("source exists");

        tokio::task::yield_now().await;
        time::advance(Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
        metrics.bytes.store(15_000, std::sync::atomic::Ordering::Release);
        time::advance(Duration::from_secs(15)).await;
        tokio::task::yield_now().await;
        assert_eq!(sampler.snapshot(), Ok(1_000));

        sampler.stop();
        tokio::task::yield_now().await;
        assert!(sampler.is_finished());
    }

    #[derive(Default)]
    struct TestMetrics {
        bytes: std::sync::atomic::AtomicU64,
    }

    impl EventMetrics for TestMetrics {
        fn transport_inbound_bytes(&self) -> u64 {
            0
        }
        fn transport_outbound_bytes(&self) -> u64 {
            0
        }
        fn transit_inbound_bytes(&self) -> u64 {
            0
        }
        fn transit_outbound_bytes(&self) -> u64 {
            self.bytes.load(std::sync::atomic::Ordering::Acquire)
        }
        fn transit_bytes_snapshot(&self) -> Option<u64> {
            Some(self.transit_outbound_bytes())
        }
        fn connected_routers(&self) -> usize {
            0
        }
        fn transit_tunnel_count(&self) -> usize {
            0
        }
        fn tunnel_build_successes(&self) -> u64 {
            0
        }
        fn tunnel_build_failures(&self) -> u64 {
            0
        }
        fn ipv4_firewall_status(&self) -> emissary_core::FirewallStatus {
            emissary_core::FirewallStatus::Unknown
        }
        fn ipv6_firewall_status(&self) -> emissary_core::FirewallStatus {
            emissary_core::FirewallStatus::Unknown
        }
    }
}
