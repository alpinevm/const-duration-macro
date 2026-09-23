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

A typo is a build failure:

```text
error[E0080]: evaluation panicked: unknown duration unit
   |
   |     const TIMEOUT: Duration = duration!("9 fortnights");
   |                               ------------------------- in this macro invocation
```

No dependencies, no proc macro, `#![no_std]`. The whole crate is one `const fn` that
walks the literal's bytes and an inline `const { … }` block that forces it to run
during compilation.

## Grammar

A literal is one or more pairs of an integer and a unit, such as `9 hrs` or `250ms`.
Spaces between a number and its unit, and between pairs, are optional. Repeated units
add up.

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

The grammar is modelled on [humantime](https://crates.io/crates/humantime), and
`tests/humantime.rs` checks against humantime 2.4 that every literal both accept parses
to the same `Duration`. The differences are deliberate:

- months and years: humantime uses 30.44 and 365.25 days, this crate the Gregorian means
  above;
- decimals: humantime accepts `1.5h`, this crate does not (write `90m`);
- this crate additionally accepts the full words `nanosecond(s)`, `microsecond(s)` and
  `millisecond(s)`, the Greek mu in `μs`, and integers wider than `u64`;
- when nanoseconds carry past `u64::MAX` seconds, as in `18446744073709551615s 1000000000ns`,
  this crate reports overflow where humantime 2.4 panics in `Duration::new`.

## MSRV

**Rust 1.79.**

CI runs the test suite on both 1.79 and current stable.

## License

MIT or Apache-2.0
