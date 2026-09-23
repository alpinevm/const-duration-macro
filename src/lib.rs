//! Human-readable [`Duration`] literals, parsed at compile time.
//!
//! ```
//! use core::time::Duration;
//! use const_duration_macro::duration;
//!
//! const NINE_HOURS: Duration = duration!("9 hrs");
//!
//! assert_eq!(NINE_HOURS, Duration::from_secs(9 * 3600));
//! assert_eq!(duration!("10 secs"), Duration::from_secs(10));
//! assert_eq!(duration!("1h 30m"), Duration::from_secs(5400));
//! assert_eq!(duration!("250ms"), Duration::from_millis(250));
//! assert_eq!(duration!("1500us"), Duration::from_micros(1500));
//! ```
//!
//! A literal that does not parse is a compile error, not a runtime panic:
//!
//! ```compile_fail,E0080
//! use const_duration_macro::duration;
//!
//! let _ = duration!("9 fortnights");
//! ```
//!
//! # Grammar
//!
//! A literal is one or more pairs of an integer and a unit, such as `9 hrs` or `250ms`.
//! Spaces between a number and its unit, and between pairs, are optional. Repeated units
//! add up. The units are:
//!
//! | unit                                              | means            |
//! |---------------------------------------------------|------------------|
//! | `ns`, `nsec`, `nanosecond`, `nanoseconds`         | nanoseconds      |
//! | `us`, `µs`, `usec`, `microsecond`, `microseconds` | microseconds     |
//! | `ms`, `msec`, `millisecond`, `milliseconds`       | milliseconds     |
//! | `s`, `sec`, `secs`, `second`, `seconds`           | seconds          |
//! | `m`, `min`, `mins`, `minute`, `minutes`           | minutes          |
//! | `h`, `hr`, `hrs`, `hour`, `hours`                 | hours            |
//! | `d`, `day`, `days`                                | days, 24 hours   |
//! | `w`, `wk`, `wks`, `week`, `weeks`                 | weeks, 7 days    |
//! | `M`, `month`, `months`                            | months, 30.436875 days |
//! | `y`, `yr`, `yrs`, `year`, `years`                 | years, 365.2425 days   |
//!
//! Units are matched exactly and case-sensitively; `M` is months and `m` is minutes, as
//! in humantime and systemd. `µs` is accepted with either the micro sign (U+00B5) or the
//! Greek small mu (U+03BC). Anything else, including an empty literal, a number without a
//! unit, or a total larger than [`Duration::MAX`], fails to compile.
//!
//! Months and years are mean lengths. A year is 365.2425 days, the average over the
//! Gregorian calendar's 400-year cycle (31 556 952 seconds), and a month is one twelfth
//! of that, 30.436875 days (2 629 746 seconds), so twelve months make exactly one year.

#![no_std]

use core::time::Duration;

const NANOSECOND: u64 = 1;
const MICROSECOND: u64 = 1_000 * NANOSECOND;
const MILLISECOND: u64 = 1_000 * MICROSECOND;
const SECOND: u64 = 1_000 * MILLISECOND;
const MINUTE: u64 = 60 * SECOND;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
const WEEK: u64 = 7 * DAY;
const YEAR: u64 = (400 * 365 + 97) * DAY / 400;
const MONTH: u64 = YEAR / 12;

const UNITS: &[(&str, u64)] = &[
    ("ns", NANOSECOND),
    ("nsec", NANOSECOND),
    ("nanosecond", NANOSECOND),
    ("nanoseconds", NANOSECOND),
    ("us", MICROSECOND),
    ("µs", MICROSECOND),
    ("μs", MICROSECOND),
    ("usec", MICROSECOND),
    ("microsecond", MICROSECOND),
    ("microseconds", MICROSECOND),
    ("ms", MILLISECOND),
    ("msec", MILLISECOND),
    ("millisecond", MILLISECOND),
    ("milliseconds", MILLISECOND),
    ("s", SECOND),
    ("sec", SECOND),
    ("secs", SECOND),
    ("second", SECOND),
    ("seconds", SECOND),
    ("m", MINUTE),
    ("min", MINUTE),
    ("mins", MINUTE),
    ("minute", MINUTE),
    ("minutes", MINUTE),
    ("h", HOUR),
    ("hr", HOUR),
    ("hrs", HOUR),
    ("hour", HOUR),
    ("hours", HOUR),
    ("d", DAY),
    ("day", DAY),
    ("days", DAY),
    ("w", WEEK),
    ("wk", WEEK),
    ("wks", WEEK),
    ("week", WEEK),
    ("weeks", WEEK),
    ("M", MONTH),
    ("month", MONTH),
    ("months", MONTH),
    ("y", YEAR),
    ("yr", YEAR),
    ("yrs", YEAR),
    ("year", YEAR),
    ("years", YEAR),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Error {
    Empty,
    ExpectedNumber,
    ExpectedUnit,
    UnknownUnit,
    Overflow,
}

const fn is_unit_byte(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || !byte.is_ascii()
}

const fn is_unit(b: &[u8], start: usize, end: usize, name: &str) -> bool {
    let name = name.as_bytes();
    if name.len() != end - start {
        return false;
    }
    let mut k = 0;
    while k < name.len() && name[k] == b[start + k] {
        k += 1;
    }
    k == name.len()
}

const fn unit_nanos(b: &[u8], start: usize, end: usize) -> Result<u64, Error> {
    let mut u = 0;
    while u < UNITS.len() {
        let (name, nanos) = UNITS[u];
        if is_unit(b, start, end, name) {
            return Ok(nanos);
        }
        u += 1;
    }
    Err(Error::UnknownUnit)
}

const fn mul_add(a: u128, b: u128, c: u128) -> Result<u128, Error> {
    match a.checked_mul(b) {
        Some(product) => match product.checked_add(c) {
            Some(sum) => Ok(sum),
            None => Err(Error::Overflow),
        },
        None => Err(Error::Overflow),
    }
}

const fn try_parse(b: &[u8]) -> Result<Duration, Error> {
    let mut nanos: u128 = 0;
    let mut i = 0;
    let mut parts = 0;
    while i < b.len() {
        while i < b.len() && b[i] == b' ' {
            i += 1;
        }
        if i == b.len() {
            break;
        }
        let digits = i;
        let mut n: u128 = 0;
        while i < b.len() && b[i].is_ascii_digit() {
            n = match mul_add(n, 10, (b[i] - b'0') as u128) {
                Ok(n) => n,
                Err(e) => return Err(e),
            };
            i += 1;
        }
        if i == digits {
            return Err(Error::ExpectedNumber);
        }
        while i < b.len() && b[i] == b' ' {
            i += 1;
        }
        let unit = i;
        while i < b.len() && is_unit_byte(b[i]) {
            i += 1;
        }
        if i == unit {
            return Err(Error::ExpectedUnit);
        }
        let per = match unit_nanos(b, unit, i) {
            Ok(per) => per,
            Err(e) => return Err(e),
        };
        nanos = match mul_add(n, per as u128, nanos) {
            Ok(nanos) => nanos,
            Err(e) => return Err(e),
        };
        parts += 1;
    }
    if parts == 0 {
        return Err(Error::Empty);
    }
    let secs = nanos / SECOND as u128;
    if secs > u64::MAX as u128 {
        return Err(Error::Overflow);
    }
    Ok(Duration::new(secs as u64, (nanos % SECOND as u128) as u32))
}

#[doc(hidden)]
pub const fn parse_duration(s: &str) -> Duration {
    match try_parse(s.as_bytes()) {
        Ok(duration) => duration,
        Err(Error::Empty) => panic!("empty duration"),
        Err(Error::ExpectedNumber) => panic!("expected a number"),
        Err(Error::ExpectedUnit) => panic!("expected a unit"),
        Err(Error::UnknownUnit) => panic!("unknown duration unit"),
        Err(Error::Overflow) => panic!("duration exceeds Duration::MAX"),
    }
}

/// Builds a [`Duration`] from a string literal at compile time.
///
/// ```
/// # use core::time::Duration;
/// # use const_duration_macro::duration;
/// assert_eq!(duration!("2 days"), Duration::from_secs(172_800));
/// ```
#[macro_export]
macro_rules! duration {
    ($s:literal) => {
        const { $crate::parse_duration($s) }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_spelling() {
        let cases: &[(&str, Duration)] = &[
            ("250ns", Duration::from_nanos(250)),
            ("250 nsec", Duration::from_nanos(250)),
            ("250 nanosecond", Duration::from_nanos(250)),
            ("250 nanoseconds", Duration::from_nanos(250)),
            ("250us", Duration::from_micros(250)),
            ("250µs", Duration::from_micros(250)),
            ("250μs", Duration::from_micros(250)),
            ("250 usec", Duration::from_micros(250)),
            ("250 microsecond", Duration::from_micros(250)),
            ("250 microseconds", Duration::from_micros(250)),
            ("250ms", Duration::from_millis(250)),
            ("250 msec", Duration::from_millis(250)),
            ("250 millisecond", Duration::from_millis(250)),
            ("250 milliseconds", Duration::from_millis(250)),
            ("10 s", Duration::from_secs(10)),
            ("10 sec", Duration::from_secs(10)),
            ("10 secs", Duration::from_secs(10)),
            ("10 second", Duration::from_secs(10)),
            ("10 seconds", Duration::from_secs(10)),
            ("3 m", Duration::from_secs(180)),
            ("3 min", Duration::from_secs(180)),
            ("3 mins", Duration::from_secs(180)),
            ("3 minute", Duration::from_secs(180)),
            ("3 minutes", Duration::from_secs(180)),
            ("9 h", Duration::from_secs(32_400)),
            ("9 hr", Duration::from_secs(32_400)),
            ("9 hrs", Duration::from_secs(32_400)),
            ("9 hour", Duration::from_secs(32_400)),
            ("9 hours", Duration::from_secs(32_400)),
            ("2 d", Duration::from_secs(172_800)),
            ("2 day", Duration::from_secs(172_800)),
            ("2 days", Duration::from_secs(172_800)),
            ("2 w", Duration::from_secs(1_209_600)),
            ("2 wk", Duration::from_secs(1_209_600)),
            ("2 wks", Duration::from_secs(1_209_600)),
            ("2 week", Duration::from_secs(1_209_600)),
            ("2 weeks", Duration::from_secs(1_209_600)),
            ("1M", Duration::from_secs(2_629_746)),
            ("1 month", Duration::from_secs(2_629_746)),
            ("1 months", Duration::from_secs(2_629_746)),
            ("1 y", Duration::from_secs(31_556_952)),
            ("1 yr", Duration::from_secs(31_556_952)),
            ("1 yrs", Duration::from_secs(31_556_952)),
            ("1 year", Duration::from_secs(31_556_952)),
            ("1 years", Duration::from_secs(31_556_952)),
        ];
        for (literal, expected) in cases {
            assert_eq!(parse_duration(literal), *expected, "{literal}");
        }
    }

    #[test]
    fn compound_and_spacing() {
        assert_eq!(parse_duration("1h 30m"), Duration::from_secs(5400));
        assert_eq!(parse_duration("1h30m"), Duration::from_secs(5400));
        assert_eq!(parse_duration("  1 h   30 m  "), Duration::from_secs(5400));
        assert_eq!(
            parse_duration("1y 1w 1d 2h 3m 4s 5ms"),
            Duration::from_millis(31_556_952_000 + 604_800_000 + 93_784_005)
        );
        assert_eq!(parse_duration("12 M"), parse_duration("1 y"));
        assert_eq!(
            parse_duration("1s 500ms 250us 1ns"),
            Duration::new(1, 500_250_001)
        );
        assert_eq!(parse_duration("30m 1h"), Duration::from_secs(5400));
        assert_eq!(parse_duration("1h 1h"), Duration::from_secs(7200));
    }

    #[test]
    fn zero_and_leading_zeros() {
        assert_eq!(parse_duration("0s"), Duration::ZERO);
        assert_eq!(parse_duration("007 s"), Duration::from_secs(7));
    }

    #[test]
    fn largest_representable() {
        assert_eq!(
            parse_duration("18446744073709551615 s 999999999 ns"),
            Duration::MAX
        );
        assert_eq!(
            parse_duration("18446744073709551615999999999 ns"),
            Duration::MAX
        );
        assert_eq!(
            parse_duration("18446744073709551615 s 999 ms 999 us 999 ns"),
            Duration::MAX
        );
    }

    #[test]
    #[should_panic(expected = "duration exceeds Duration::MAX")]
    fn rejects_seconds_overflow() {
        parse_duration("18446744073709551616 s");
    }

    #[test]
    #[should_panic(expected = "duration exceeds Duration::MAX")]
    fn rejects_carry_overflow() {
        parse_duration("18446744073709551615 s 1000000000 ns");
    }

    #[test]
    #[should_panic(expected = "duration exceeds Duration::MAX")]
    fn rejects_product_overflow() {
        parse_duration("1000000000000000000000000 y");
    }

    #[test]
    #[should_panic(expected = "duration exceeds Duration::MAX")]
    fn rejects_number_overflow() {
        parse_duration("1000000000000000000000000000000000000000 ns");
    }

    #[test]
    #[should_panic(expected = "unknown duration unit")]
    fn rejects_unknown_unit() {
        parse_duration("9 fortnights");
    }

    #[test]
    #[should_panic(expected = "unknown duration unit")]
    fn rejects_case_variants() {
        parse_duration("9 Hrs");
    }

    #[test]
    #[should_panic(expected = "unknown duration unit")]
    fn rejects_truncated_unit() {
        parse_duration("9 hou");
    }

    #[test]
    #[should_panic(expected = "unknown duration unit")]
    fn rejects_other_non_ascii_unit() {
        parse_duration("9 ſ");
    }

    #[test]
    #[should_panic(expected = "expected a unit")]
    fn rejects_bare_number() {
        parse_duration("9");
    }

    #[test]
    #[should_panic(expected = "expected a number")]
    fn rejects_bare_unit() {
        parse_duration("hrs");
    }

    #[test]
    #[should_panic(expected = "expected a number")]
    fn rejects_negative() {
        parse_duration("-9 hrs");
    }

    #[test]
    #[should_panic(expected = "expected a unit")]
    fn rejects_fraction() {
        parse_duration("1.5 h");
    }

    #[test]
    #[should_panic(expected = "expected a number")]
    fn rejects_separator() {
        parse_duration("1h, 30m");
    }

    #[test]
    #[should_panic(expected = "empty duration")]
    fn rejects_empty() {
        parse_duration("");
    }

    #[test]
    #[should_panic(expected = "empty duration")]
    fn rejects_whitespace_only() {
        parse_duration("   ");
    }
}
