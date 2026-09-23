# const-duration-macro

Human-readable `Duration` literals, parsed at compile time.

```rust
use core::time::Duration;
use const_duration_macro::duration;

const NINE_HOURS: Duration = duration!("9 hrs");

assert_eq!(duration!("10 secs"), Duration::from_secs(10));
assert_eq!(duration!("1h 30m"), Duration::from_secs(5400));
assert_eq!(duration!("250ms"), Duration::from_millis(250));
assert_eq!(duration!("1500us"), Duration::from_micros(1500));
```

A typo is a compile error, reported by `cargo check` too:

```text
error[E0080]: evaluation panicked: unknown duration unit
   |
   |     const TIMEOUT: Duration = duration!("9 fortnights");
   |                               ------------------------- in this macro invocation
```

No dependencies, no proc macro, `#![no_std]`. The parser is a few `const fn`s over the
literal's bytes; the macro binds the result to a `const` item, which rustc evaluates
during analysis, so a bad literal is reported by `cargo check` and rust-analyzer, even
inside generic code that is never instantiated.

## Grammar

A literal is one or more pairs of an integer and a unit, such as `9 hrs` or `250ms`.
Spaces, meaning U+0020 only, may appear before, between and after the pairs and between
a number and its unit. Repeated units add up.

| unit                                              | means           |
|---------------------------------------------------|-----------------|
| `ns`, `nsec`, `nanosecond`, `nanoseconds`         | nanoseconds     |
| `us`, `µs`, `usec`, `microsecond`, `microseconds` | microseconds    |
| `ms`, `msec`, `millisecond`, `milliseconds`       | milliseconds    |
| `s`, `sec`, `secs`, `second`, `seconds`           | seconds         |
| `m`, `min`, `mins`, `minute`, `minutes`           | minutes         |
| `h`, `hr`, `hrs`, `hour`, `hours`                 | hours           |
| `d`, `day`, `days`                                | days, 24 hours  |
| `w`, `wk`, `wks`, `week`, `weeks`                 | weeks, 7 days   |
| `M`, `month`, `months`                            | months, 30.436875 days |
| `y`, `yr`, `yrs`, `year`, `years`                 | years, 365.2425 days   |

Units are matched exactly and case-sensitively. `M` is months and `m` is minutes, as in
humantime and systemd. `µs` is accepted with either the micro sign (U+00B5) or the Greek
small mu (U+03BC). Anything else fails to compile, including an empty literal, a number
without a unit, a unit without a number, fractions, signs, and a total larger than
`Duration::MAX`.

Months and years are mean lengths. A year is 365.2425 days, the average over the
Gregorian calendar's 400-year cycle (31 556 952 seconds), and a month is one twelfth of
that, 30.436875 days (2 629 746 seconds), so twelve months make exactly one year.

## humantime

The grammar is modelled on [humantime](https://crates.io/crates/humantime).
`tests/humantime.rs` checks against humantime 2.4.0 that every shared unit spelling, over
a range of magnitudes and spacings, parses to the same `Duration`, and pins each known
difference:

- months and years: humantime uses 30.44 and 365.25 days, this crate the Gregorian means
  above;
- humantime also accepts decimals (`1.5h`), the spellings `nanos` and `millis`, a bare
  `0`, tabs, newlines and other Unicode spaces, and digits split by spaces (`1 0 s` is
  ten seconds); this crate rejects all of these;
- this crate also accepts the full words `nanosecond(s)`, `microsecond(s)` and
  `millisecond(s)`, the Greek mu in `μs`, and any value up to `Duration::MAX`, where
  humantime rejects anything over `u64::MAX` nanoseconds, about 584 years;
- when nanoseconds carry past `u64::MAX` seconds, as in `18446744073709551615s 1000000000ns`,
  this crate reports overflow where humantime 2.4.0 panics in `Duration::new`.

## MSRV

**Rust 1.58.**

CI builds the crate on 1.58 and runs the test suite on current stable.

## License

MIT or Apache-2.0
