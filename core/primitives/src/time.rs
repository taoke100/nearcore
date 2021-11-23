use std::default::Default;

use chrono;

pub use chrono::Utc;
pub use std::time::{Duration, Instant};

use chrono::DateTime;
use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(Default)]
struct MockClockPerState {
    /// List of timestamp, we will return one timestamp to each call.
    utc: VecDeque<DateTime<Utc>>,
    /// Number of times `utc()` method was called since we started mocking.
    utc_call_count: u64,
}

/// Stores the mocking state.
#[derive(Default)]
struct MockClockPerThread {
    mock: Option<MockClockPerState>,
}

impl MockClockPerThread {
    fn with<F, T>(f: F) -> T
    where
        F: FnOnce(&mut MockClockPerThread) -> T,
    {
        thread_local! {
            static INSTANCE: RefCell<MockClockPerThread> = RefCell::default()
        }
        INSTANCE.with(|it| f(&mut *it.borrow_mut()))
    }
}

pub struct MockClockGuard {}

impl MockClockGuard {
    /// Adds timestamp to queue, it will be returned in `Self::utc()`.
    pub fn add_utc(&self, mock_date: DateTime<chrono::Utc>) {
        MockClockPerThread::with(|clock| match &mut clock.mock {
            Some(clock) => {
                clock.utc.push_back(mock_date);
            }
            None => {
                panic!("Use MockClockGuard in your test");
            }
        });
    }

    /// Returns number of calls  to `Self::utc` since `Self::mock()` was called.
    pub fn utc_call_count(&self) -> u64 {
        MockClockPerThread::with(|clock| match &mut clock.mock {
            Some(clock) => clock.utc_call_count,
            None => {
                panic!("Use MockClockGuard in your test");
            }
        })
    }
}

impl Default for MockClockGuard {
    fn default() -> Self {
        Clock::set_mock();
        Self {}
    }
}

impl Drop for MockClockGuard {
    fn drop(&mut self) {
        Clock::reset();
    }
}

pub struct Clock {}

impl Clock {
    /// Turns the mocking logic on.
    fn set_mock() {
        MockClockPerThread::with(|clock| clock.mock = Some(MockClockPerState::default()))
    }

    /// Resets mocks to default state.
    fn reset() {
        MockClockPerThread::with(|clock| clock.mock = None);
    }

    /// Gets mocked instant.
    pub fn instant() -> Instant {
        MockClockPerThread::with(|clock| match &mut clock.mock {
            Some(_clock) => panic!("Mock clock run out of samples"),
            None => Instant::now(),
        })
    }

    /// Returns time pushed by `Self::push_utc()`
    pub fn utc() -> DateTime<chrono::Utc> {
        MockClockPerThread::with(|clock| match &mut clock.mock {
            Some(clock) => {
                clock.utc_call_count += 1;
                let x = clock.utc.pop_front();
                match x {
                    Some(t) => t,
                    None => {
                        panic!("Mock clock run out of samples");
                    }
                }
            }
            None => chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_clock_panic_utc() {
        let _mock_clock_guard = MockClockGuard::default();
        Clock::utc();
    }

    #[test]
    #[should_panic]
    fn test_clock_panic_instant() {
        let _mock_clock_guard = MockClockGuard::default();
        Clock::instant();
    }

    #[test]
    fn test_clock() {
        {
            let mock_clock_guard = MockClockGuard::default();

            let utc_now = Utc::now();
            mock_clock_guard.add_utc(
                utc_now
                    .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(1)).unwrap())
                    .unwrap(),
            );
            mock_clock_guard.add_utc(
                utc_now
                    .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(2)).unwrap())
                    .unwrap(),
            );
            mock_clock_guard.add_utc(
                utc_now
                    .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(3)).unwrap())
                    .unwrap(),
            );
            assert_eq!(
                Clock::utc(),
                utc_now
                    .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(1)).unwrap())
                    .unwrap(),
            );
            assert_eq!(
                Clock::utc(),
                utc_now
                    .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(2)).unwrap())
                    .unwrap(),
            );
            assert_eq!(
                Clock::utc(),
                utc_now
                    .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(3)).unwrap())
                    .unwrap(),
            );

            assert_eq!(mock_clock_guard.utc_call_count(), 3);
            drop(mock_clock_guard);
        }
        let mock_clock_guard = MockClockGuard::default();
        assert_eq!(mock_clock_guard.utc_call_count(), 0);
    }
}
