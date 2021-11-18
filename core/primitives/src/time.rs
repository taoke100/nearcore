// TODO#(5174) don't export external types from your own crate
pub use chrono::Utc;
pub use clock::{Clock, MockClockGuard};
// TODO#(5174) don't export external types from your own crate
pub use std::time::{Duration, Instant};
pub use time::Time;

mod clock {
    use crate::time::{Duration, Instant, Time, Utc};
    use chrono;
    use chrono::DateTime;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::default::Default;

    struct MockClockPerThread {
        utc: VecDeque<DateTime<Utc>>,
        durations: VecDeque<Duration>,
        utc_call_count: u64,
        instant_call_count: u64,
        instant: Instant,
        is_mock: bool,
    }

    pub struct Clock {}

    impl MockClockPerThread {
        pub fn reset(&mut self) {
            self.utc.clear();
            self.durations.clear();
            self.utc_call_count = 0;
            self.instant_call_count = 0;
            self.instant = Instant::now();
            self.is_mock = false;
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
        fn pop_instant(&mut self) -> Option<Instant> {
            self.instant_call_count += 1;
            let x = self.durations.pop_front();
            match x {
                Some(t) => self.instant.checked_add(t),
                None => None,
            }
        }
    }

    impl Default for MockClockPerThread {
        fn default() -> Self {
            Self {
                utc: VecDeque::with_capacity(16),
                durations: VecDeque::with_capacity(16),
                utc_call_count: 0,
                instant_call_count: 0,
                instant: Instant::now(),
                is_mock: false,
            }
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
                clock.is_mock = true;
            });
        }
        fn reset() {
            MockClockPerThread::with(|clock| {
                clock.reset();
            });
        }
        pub fn add_utc(mock_date: DateTime<chrono::Utc>) {
            MockClockPerThread::with(|clock| {
                if clock.is_mock {
                    clock.utc.push_back(mock_date);
                } else {
                    panic!("Use MockClockGuard in your test");
                }
            });
        }

        pub fn add_instant(mock_instant: Duration) {
            MockClockPerThread::with(|clock| {
                if clock.is_mock {
                    clock.durations.push_back(mock_instant);
                } else {
                    panic!("Use MockClockGuard in your test");
                }
            });
        }

        pub fn utc() -> DateTime<chrono::Utc> {
            MockClockPerThread::with(|clock| {
                if clock.is_mock {
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

        pub fn instant() -> Instant {
            MockClockPerThread::with(|clock| {
                if clock.is_mock {
                    let x = clock.pop_instant();
                    match x {
                        Some(t) => t,
                        None => {
                            panic!("Mock clock run out of samples");
                        }
                    }
                } else {
                    Instant::now()
                }
            })
        }

        pub fn now() -> Time {
            Clock::utc().into()
        }

        pub fn instant_call_count() -> u64 {
            MockClockPerThread::with(|clock| clock.instant_call_count)
        }

        pub fn utc_call_count() -> u64 {
            MockClockPerThread::with(|clock| clock.utc_call_count)
        }
    }

    #[cfg(test)]
    mod tests {
        // TODO(#5345) Add tests for mock
        use super::*;

        #[test]
        fn test_mock() {
            Clock::set_mock();
            let utc = Utc::now();

            Clock::add_utc(utc);

            assert_eq!(Clock::utc(), utc);
        }
    }
}

mod time {
    use crate::time::{Clock, Duration, Utc};
    use borsh::{BorshDeserialize, BorshSerialize};
    use chrono::DateTime;
    use std::ops::{Add, Sub};
    use std::time::SystemTime;

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct Time {
        system_time: SystemTime,
    }

    impl BorshSerialize for Time {
        fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
            let nanos = self.to_unix_timestamp_nanos().as_nanos() as u64;
            BorshSerialize::serialize(&nanos, writer).unwrap();
            Ok(())
        }
    }

    impl BorshDeserialize for Time {
        fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
            let nanos: u64 = borsh::BorshDeserialize::deserialize(buf)?;

            Ok(Time::from_unix_timestamp(Duration::from_nanos(nanos)))
        }
    }

    impl From<SystemTime> for Time {
        fn from(system_time: SystemTime) -> Self {
            Self { system_time }
        }
    }

    impl From<DateTime<Utc>> for Time {
        fn from(utc: DateTime<Utc>) -> Self {
            // utc.timestamp_nanos() returns i64
            let nanos = utc.timestamp_nanos() as u64;

            Self::UNIX_EPOCH + Duration::from_nanos(nanos)
        }
    }

    impl Time {
        pub const UNIX_EPOCH: Time = Time { system_time: SystemTime::UNIX_EPOCH };

        pub fn now() -> Self {
            Self::UNIX_EPOCH + Duration::from_nanos(Clock::utc().timestamp_nanos() as u64)
        }

        pub fn duration_since(&self, rhs: &Self) -> Duration {
            self.system_time.duration_since(rhs.system_time).unwrap_or(Duration::from_millis(0))
        }

        pub fn elapsed(&self) -> Duration {
            Self::now().duration_since(self)
        }

        pub fn from_unix_timestamp(duration: Duration) -> Self {
            Self::UNIX_EPOCH + duration
        }

        pub fn to_unix_timestamp_nanos(&self) -> Duration {
            self.duration_since(&Self::UNIX_EPOCH)
        }

        pub fn inner(self) -> SystemTime {
            self.system_time
        }
    }

    impl Add<Duration> for Time {
        type Output = Self;

        fn add(self, other: Duration) -> Self {
            Self { system_time: self.system_time + other }
        }
    }

    impl Sub for Time {
        type Output = Duration;

        fn sub(self, other: Self) -> Self::Output {
            self.system_time.duration_since(other.system_time).unwrap_or(Duration::from_millis(0))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use borsh::{BorshDeserialize, BorshSerialize};

        #[test]
        fn test_operator() {
            let now_st = SystemTime::now();

            let now_nc: Time = now_st.into();

            let t_nc = now_nc + Duration::from_nanos(123456);
            let t_st = now_st + Duration::from_nanos(123456);

            assert_eq!(t_nc.inner(), t_st);
        }

        #[test]
        fn test_borsh() {
            let now_nc = Time::now();

            let mut v = Vec::new();
            BorshSerialize::serialize(&now_nc, &mut v).unwrap();

            let v2: &mut &[u8] = &mut v.as_slice();

            let now2: Time = BorshDeserialize::deserialize(v2).unwrap();
            assert_eq!(now_nc, now2);
        }
    }
}
