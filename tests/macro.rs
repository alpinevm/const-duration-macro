#![no_std]

use const_duration_macro::duration;
use core::time::Duration;

const NINE_HOURS: Duration = duration!("9 hrs");
static TEN_SECONDS: Duration = duration!("10 secs");

#[test]
fn usable_in_const_and_static_items() {
    assert_eq!(NINE_HOURS, Duration::from_secs(32_400));
    assert_eq!(TEN_SECONDS, Duration::from_secs(10));
}

#[test]
fn usable_as_an_expression() {
    assert_eq!(duration!("1h 30m"), Duration::from_secs(5400));
    assert_eq!(duration!("250ms"), Duration::from_millis(250));
    assert_eq!(duration!("2 days"), Duration::from_secs(172_800));
}

#[test]
fn usable_in_const_generics() {
    struct Every<const MS: u128>;
    let _: Every<{ duration!("1 min").as_millis() }> = Every::<60_000>;
}
