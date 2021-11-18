use std::default::Default;

use chrono;

pub use chrono::Utc;
pub use std::time::{Duration, Instant};

use chrono::DateTime;
use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(Default)]
struct MockClockPerThread {
    utc: VecDeque<DateTime<Utc>>,
    utc_call_count: u64,
    is_mocked: bool,
}

pub struct Clock {}

impl MockClockPerThread {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn with<F, T>(f: F) -> T
    where
        F: FnOnce(&mut MockClockPerThread) -> T,
    {
        thread_local! {
            static INSTANCE: RefCell<MockClockPerThread> = RefCell::default()
        }
        INSTANCE.with(|it| f(&mut *it.borrow_mut()))
    }

    fn pop_utc(&mut self) -> Option<DateTime<chrono::Utc>> {
        self.utc_call_count += 1;
        self.utc.pop_front()
    }
}

pub struct MockClockGuard {}

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

impl Clock {
    fn set_mock() {
        MockClockPerThread::with(|clock| {
            clock.is_mocked = true;
        });
    }

    fn reset() {
        MockClockPerThread::with(|clock| {
            clock.reset();
        });
    }

    pub fn add_utc(mock_date: DateTime<chrono::Utc>) {
        MockClockPerThread::with(|clock| {
            if clock.is_mocked {
                clock.utc.push_back(mock_date);
            } else {
                panic!("Use MockClockGuard in your test");
            }
        });
    }

    pub fn instant() -> Instant {
        MockClockPerThread::with(|clock| {
            if clock.is_mocked {
                panic!("Mock clock run out of samples");
            } else {
                Instant::now()
            }
        })
    }

    pub fn utc() -> DateTime<chrono::Utc> {
        MockClockPerThread::with(|clock| {
            if clock.is_mocked {
                let x = clock.pop_utc();
                match x {
                    Some(t) => t,
                    None => {
                        panic!("Mock clock run out of samples");
                    }
                }
            } else {
                chrono::Utc::now()
            }
        })
    }

    pub fn utc_call_count() -> u64 {
        MockClockPerThread::with(|clock| clock.utc_call_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock() {
        Clock::set_mock();
        let utc_now = Utc::now();
        Clock::add_utc(
            utc_now
                .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(1)).unwrap())
                .unwrap(),
        );
        Clock::add_utc(
            utc_now
                .checked_add_signed(chrono::Duration::from_std(Duration::from_secs(2)).unwrap())
                .unwrap(),
        );
        Clock::add_utc(
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

        assert_eq!(Clock::utc_call_count(), 3);
        Clock::reset();
        assert_eq!(Clock::utc_call_count(), 0);
    }
}
