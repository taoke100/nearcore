use std::default::Default;

use chrono;

pub use chrono::Utc;
pub use std::time::{Duration, Instant};

use chrono::DateTime;
use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(Default)]
// stores mocking state
struct MockClockPerThread {
    // list of mocks, they will be returned each time call to get current time is made
    utc: VecDeque<DateTime<Utc>>,
    // number of times `utc()` method was called since we started mocking
    utc_call_count: u64,
    // false for default behaviour
    // true when we want to enable mocking logic
    is_mocked: bool,
}

pub struct Clock {}

thread_local! {
    static INSTANCE: RefCell<MockClockPerThread> = RefCell::default()
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
    // turns on mocking logic
    fn set_mock() {
        INSTANCE.with(|clock| {
            let clock = &mut clock.borrow_mut();
            clock.is_mocked = true;
        });
    }

    // removes mock
    fn reset() {
        INSTANCE.with(|clock| *clock.borrow_mut() = MockClockPerThread::default());
    }

    // adds timestamp to queue, it will be returned in `Self::utc()`
    pub fn add_utc(mock_date: DateTime<chrono::Utc>) {
        INSTANCE.with(|clock| {
            let clock = &mut clock.borrow_mut();
            if clock.is_mocked {
                clock.utc.push_back(mock_date);
            } else {
                panic!("Use MockClockGuard in your test");
            }
        });
    }

    // gets mocked instant
    pub fn instant() -> Instant {
        INSTANCE.with(|clock| {
            let clock = clock.borrow();
            if clock.is_mocked {
                panic!("Mock clock run out of samples");
            } else {
                Instant::now()
            }
        })
    }

    // returns time pushed by `Self::push_utc()`
    pub fn utc() -> DateTime<chrono::Utc> {
        INSTANCE.with(|clock| {
            let clock = &mut clock.borrow_mut();
            if clock.is_mocked {
                clock.utc_call_count += 1;
                let x = clock.utc.pop_front();
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

    // returns number of calls  to `Self::utc` since `Self::mock()` was called.
    pub fn utc_call_count() -> u64 {
        INSTANCE.with(|clock| clock.borrow().utc_call_count)
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
