//! Ray-level scheduling only. No numerical state or per-step locks are shared.
use std::{sync::mpsc, thread, time::Duration};

#[derive(Clone, Copy, Debug)]
pub enum RayExecution {
    Serial,
    Parallel { threads: usize },
}
impl RayExecution {
    /// Reserve one logical CPU for UI/OS by default. This is an execution policy,
    /// not a physical parameter: it is intentionally absent from common.json.
    pub fn from_environment() -> Result<Self, String> {
        let threads = match std::env::var("KERR_RAY_THREADS") {
            Ok(value) => value
                .parse::<usize>()
                .map_err(|_| "KERR_RAY_THREADS must be a positive integer")?,
            Err(std::env::VarError::NotPresent) => Self::default_threads(),
            Err(_) => return Err("KERR_RAY_THREADS is not valid Unicode".into()),
        };
        if threads == 0 {
            return Err("KERR_RAY_THREADS must be positive".into());
        }
        Ok(Self::Parallel { threads })
    }
    pub fn default_threads() -> usize {
        thread::available_parallelism().map_or(1, |n| n.get().saturating_sub(1).max(1))
    }
}

/// Coordinator owns the FnMut callback (no new Send/Sync requirement). At most
/// one ray per worker is dispatched; workers cannot fetch ahead by themselves.
/// Cancellation stops new dispatch immediately and joins the already-running
/// whole-ray solves. Completed partial data is never returned as a complete run.
pub(crate) fn ordered_map<T: Send, R: Send>(
    items: Vec<T>,
    execution: RayExecution,
    operation: impl Fn(T) -> R + Sync,
    mut keep_going: impl FnMut(usize, usize) -> bool,
) -> Result<Vec<R>, String> {
    let count = items.len();
    let threads = match execution {
        RayExecution::Serial => 1,
        RayExecution::Parallel { threads: 0 } => {
            return Err("ray thread count must be positive".into());
        }
        RayExecution::Parallel { threads } => threads.min(count.max(1)),
    };
    if !keep_going(0, count) {
        return Err("cancelled before first ray".into());
    }
    if threads == 1 {
        let mut out = Vec::with_capacity(count);
        for item in items {
            out.push(operation(item));
            if !keep_going(out.len(), count) {
                return Err("cancelled after ray".into());
            }
        }
        return Ok(out);
    }
    thread::scope(|scope| {
        let (finished_tx, finished_rx) = mpsc::channel();
        let mut jobs = Vec::with_capacity(threads);
        for worker in 0..threads {
            let (tx, rx) = mpsc::sync_channel::<(usize, T)>(1);
            let finished_tx = finished_tx.clone();
            let operation = &operation;
            thread::Builder::new()
                .name(format!("kerr-ray-{worker}"))
                .spawn_scoped(scope, move || {
                    while let Ok((index, item)) = rx.recv() {
                        // A worker panic must not leave the coordinator waiting on
                        // an idle pool forever. This is not a NumericalFailure ray.
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            operation(item)
                        }))
                        .map_err(|_| "ray worker panicked".to_string());
                        if finished_tx.send((worker, index, result)).is_err() {
                            break;
                        }
                    }
                })
                .map_err(|e| format!("cannot spawn ray worker: {e}"))?;
            jobs.push(tx);
        }
        drop(finished_tx);
        let mut pending = items.into_iter().enumerate();
        for job in &jobs {
            if let Some(item) = pending.next() {
                job.send(item).map_err(|_| "ray worker disconnected")?;
            }
        }
        let mut ordered: Vec<Option<R>> = (0..count).map(|_| None).collect();
        let mut completed = 0;
        while completed < count {
            match finished_rx.recv_timeout(Duration::from_millis(10)) {
                Ok((worker, index, result)) => {
                    ordered[index] = Some(result?);
                    completed += 1;
                    if !keep_going(completed, count) {
                        return Err("cancelled before next ray".into());
                    }
                    if let Some(item) = pending.next() {
                        jobs[worker]
                            .send(item)
                            .map_err(|_| "ray worker disconnected")?;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !keep_going(completed, count) {
                        return Err("cancelled while rays were running".into());
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("ray workers disconnected".into());
                }
            }
        }
        drop(jobs);
        ordered
            .into_iter()
            .map(|r| r.ok_or_else(|| "missing ray result".into()))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[test]
    fn completion_order_does_not_change_output_order() {
        let result = ordered_map(
            (0..19).collect(),
            RayExecution::Parallel { threads: 4 },
            |i| {
                thread::sleep(Duration::from_millis((19 - i) as u64));
                i * i
            },
            |_, _| true,
        )
        .unwrap();
        assert_eq!(result, (0..19).map(|i| i * i).collect::<Vec<_>>());
    }
    #[test]
    fn cancellation_does_not_dispatch_beyond_inflight_workers() {
        let starts = AtomicUsize::new(0);
        let result = ordered_map(
            (0..100).collect(),
            RayExecution::Parallel { threads: 3 },
            |i| {
                starts.fetch_add(1, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(30));
                i
            },
            |done, _| done == 0,
        );
        assert!(result.unwrap_err().contains("cancelled"));
        assert_eq!(starts.load(Ordering::SeqCst), 3);
    }
    #[test]
    fn callback_can_cancel_before_any_completion_and_is_not_send() {
        let starts = AtomicUsize::new(0);
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let release = std::sync::Barrier::new(3);
        let result = ordered_map(
            (0..50).collect(),
            RayExecution::Parallel { threads: 2 },
            |i| {
                starts.fetch_add(1, Ordering::SeqCst);
                release.wait();
                i
            },
            |done, _| {
                assert_eq!(done, 0);
                calls.set(calls.get() + 1);
                if calls.get() == 1 {
                    true
                } else {
                    release.wait();
                    false
                }
            },
        );
        assert!(result.unwrap_err().contains("cancelled"));
        assert_eq!(starts.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn worker_panic_returns_error_without_deadlock() {
        let result = ordered_map(
            vec![0, 1, 2],
            RayExecution::Parallel { threads: 2 },
            |i| {
                assert_ne!(i, 0, "test panic");
                i
            },
            |_, _| true,
        );
        assert!(result.unwrap_err().contains("panicked"));
    }
}
