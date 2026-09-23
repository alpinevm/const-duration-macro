//! Golden vectors against humantime, the parser this grammar is modelled on.
//!
//! Where the two grammars overlap, sampled literals must parse to the same `Duration`.
//! Where they differ, the difference is pinned here on purpose, with both sides' values.

use const_duration_macro::parse_duration;
use core::time::Duration;
use std::panic::{self, catch_unwind};
use std::sync::Once;

const MESSAGES: &[&str] = &[
    "empty duration",
    "expected a number",
    "expected a unit",
    "unknown duration unit",
    "duration exceeds Duration::MAX",
];

fn ours(text: &str) -> Result<Duration, &'static str> {
    static QUIET: Once = Once::new();
    QUIET.call_once(|| {
        let default = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let documented = matches!(
                info.payload().downcast_ref::<&str>(),
                Some(message) if MESSAGES.contains(message)
            );
            if !documented {
                default(info);
            }
        }));
    });
    catch_unwind(|| parse_duration(text)).map_err(|payload| {
        let message = payload.downcast_ref::<&'static str>().copied();
        match message {
            Some(message) if MESSAGES.contains(&message) => message,
            _ => panic::resume_unwind(payload),
        }
    })
}

fn theirs(text: &str) -> Result<Duration, String> {
    humantime::parse_duration(text).map_err(|e| e.to_string())
}

const SHARED_UNITS: &[&str] = &[
    "ns", "nsec", "us", "µs", "usec", "ms", "msec", "s", "sec", "secs", "second", "seconds", "m",
    "min", "mins", "minute", "minutes", "h", "hr", "hrs", "hour", "hours", "d", "day", "days", "w",
    "wk", "wks", "week", "weeks",
];

const MAGNITUDES: &[&str] = &[
    "0",
    "1",
    "7",
    "007",
    "250",
    "1000",
    "65535",
    "1000000000",
    "4294967296",
];

#[test]
fn shared_units_agree() {
    for unit in SHARED_UNITS {
        for n in MAGNITUDES {
            for text in [
                format!("{n}{unit}"),
                format!("{n} {unit}"),
                format!("  {n}   {unit}  "),
            ] {
                let expected =
                    theirs(&text).unwrap_or_else(|e| panic!("humantime rejected {text:?}: {e}"));
                assert_eq!(ours(&text), Ok(expected), "{text:?}");
            }
        }
    }
}

#[test]
fn compound_literals_agree() {
    for text in [
        "2h 37min",
        "32ms",
        "1hour 12min 5s",
        "1h30m",
        "1h 1h",
        "30m 1h",
        "  1 h   30 m  ",
        "1s 500ms 250us 1ns",
        "1d 2h 3m 4s 5ms 6us 7ns",
        "999999999ns 1ns",
        "18446744073709551615 s",
        "18446744073709551615 s 999999999 ns",
    ] {
        let expected = theirs(text).unwrap_or_else(|e| panic!("humantime rejected {text:?}: {e}"));
        assert_eq!(ours(text), Ok(expected), "{text:?}");
    }
}

#[test]
fn both_reject() {
    for text in [
        "",
        "   ",
        "9",
        "hrs",
        "-9 hrs",
        "1h, 30m",
        "9 Hrs",
        "9 hou",
        "9 fortnights",
        "18446744073709551616 s",
        "18446744073709551614 s 2000000000 ns",
    ] {
        assert!(ours(text).is_err(), "{text:?} accepted here");
        assert!(theirs(text).is_err(), "{text:?} accepted by humantime");
    }
}

#[test]
fn nanosecond_carry_past_u64_seconds_is_an_error_here_and_a_panic_in_humantime() {
    for text in [
        "18446744073709551615 s 1000000000 ns",
        "18446744073709551615s 999999999ns 1ns",
    ] {
        assert_eq!(
            ours(text),
            Err("duration exceeds Duration::MAX"),
            "{text:?}"
        );
        assert!(
            catch_unwind(|| humantime::parse_duration(text)).is_err(),
            "humantime did not panic on {text:?}"
        );
    }
}

#[test]
fn months_and_years_use_different_means() {
    assert_eq!(ours("1M"), Ok(Duration::from_secs(2_629_746)));
    assert_eq!(theirs("1M"), Ok(Duration::from_secs(2_630_016)));
    assert_eq!(ours("1y"), Ok(Duration::from_secs(31_556_952)));
    assert_eq!(theirs("1y"), Ok(Duration::from_secs(31_557_600)));
    for unit in ["M", "month", "months", "y", "yr", "yrs", "year", "years"] {
        let text = format!("3 {unit}");
        assert!(ours(&text).is_ok(), "{text:?}");
        assert!(theirs(&text).is_ok(), "{text:?}");
    }
}

#[test]
fn full_word_subsecond_units_are_ours_only() {
    for (text, expected) in [
        ("250 nanosecond", Duration::from_nanos(250)),
        ("250 nanoseconds", Duration::from_nanos(250)),
        ("250 microsecond", Duration::from_micros(250)),
        ("250 microseconds", Duration::from_micros(250)),
        ("250 millisecond", Duration::from_millis(250)),
        ("250 milliseconds", Duration::from_millis(250)),
    ] {
        assert_eq!(ours(text), Ok(expected), "{text:?}");
        assert!(theirs(text).is_err(), "{text:?} accepted by humantime");
    }
}

#[test]
fn greek_mu_is_ours_only() {
    assert_eq!(ours("250µs"), Ok(Duration::from_micros(250)));
    assert_eq!(theirs("250µs"), Ok(Duration::from_micros(250)));
    assert_eq!(ours("250μs"), Ok(Duration::from_micros(250)));
    assert!(theirs("250μs").is_err());
}

#[test]
fn decimals_are_humantime_only() {
    assert_eq!(theirs("1.5 h"), Ok(Duration::from_secs(5400)));
    assert_eq!(ours("1.5 h"), Err("expected a unit"));
}

#[test]
fn humantime_only() {
    for (text, expected) in [
        ("1nanos", Duration::from_nanos(1)),
        ("1millis", Duration::from_millis(1)),
        ("0", Duration::ZERO),
        ("1\th", Duration::from_secs(3600)),
        ("1\nh", Duration::from_secs(3600)),
        ("1\u{a0}h", Duration::from_secs(3600)),
        ("1 0 s", Duration::from_secs(10)),
    ] {
        assert_eq!(theirs(text), Ok(expected), "{text:?}");
        assert!(ours(text).is_err(), "{text:?} accepted here");
    }
}

#[test]
fn values_over_u64_nanoseconds_are_ours_only() {
    let largest_shared = Duration::from_micros(18_446_744_073_709_551);
    assert_eq!(ours("18446744073709551us"), Ok(largest_shared));
    assert_eq!(theirs("18446744073709551us"), Ok(largest_shared));
    for (text, expected) in [
        (
            "18446744073709552us",
            Duration::from_micros(18_446_744_073_709_552),
        ),
        ("18446744073709551615999999999 ns", Duration::MAX),
    ] {
        assert_eq!(ours(text), Ok(expected), "{text:?}");
        assert!(theirs(text).is_err(), "{text:?} accepted by humantime");
    }
}
