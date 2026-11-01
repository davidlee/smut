//! Exact comparison of JSON numbers: integers and floats against the value each
//! actually represents.
//!
//! DEC-006 rejects the obvious implementation — integers exactly, mixed pairs as
//! `f64` — because switching comparison domain per pair is not merely imprecise,
//! it is *cyclic*: with integers `9007199254740992` and `9007199254740993` and
//! float `9007199254740992.0` it yields `z < a`, `a < m`, `m < z`. A comparator
//! that can cycle cannot sort. So every number is decomposed to one exact
//! representation — sign · integer magnitude · power of two — and both sides are
//! compared in that representation, whatever their JSON spelling.
//!
//! Two consequences are deliberate and documented rather than incidental:
//! `1` and `1.0` are unequal for structural equality (DEC-002) but compare equal
//! here, and `0`, `0.0` and `-0.0` are one value for ordering, tied apart by the
//! ID tiebreak in [`crate::order`].

use std::cmp::Ordering;

use serde_json::Number;

/// Compares two JSON numbers exactly, with no rounding of either side.
pub fn compare(left: &Number, right: &Number) -> Ordering {
    exact(left).compare(exact(right))
}

/// One number as `sign · magnitude · 2^exponent`, with an integer magnitude.
///
/// Every finite `f64` is exactly `±m · 2^e` for an integer `m` of at most 53
/// significant bits, and every integer JSON number is `±m · 2^0`, so this is a
/// lossless common form rather than an approximation.
#[derive(Clone, Copy, Debug)]
struct Exact {
    negative: bool,
    magnitude: u128,
    exponent: i32,
}

impl Exact {
    fn compare(self, other: Exact) -> Ordering {
        if self.magnitude == 0 && other.magnitude == 0 {
            return Ordering::Equal;
        }

        if self.negative != other.negative {
            return if self.negative {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        let magnitude = compare_magnitudes(self, other);

        if self.negative {
            magnitude.reverse()
        } else {
            magnitude
        }
    }
}

/// Compares two magnitudes exactly, aligning the side with the smaller exponent
/// so both are integral before they meet.
fn compare_magnitudes(left: Exact, right: Exact) -> Ordering {
    match (left.magnitude, right.magnitude) {
        (0, 0) => return Ordering::Equal,
        (0, _) => return Ordering::Less,
        (_, 0) => return Ordering::Greater,
        _ => {}
    }

    let shift = i64::from(left.exponent) - i64::from(right.exponent);
    let left_bits = i64::from(128 - left.magnitude.leading_zeros()) + shift;
    let right_bits = i64::from(128 - right.magnitude.leading_zeros());

    if left_bits != right_bits {
        return left_bits.cmp(&right_bits);
    }

    // Equal bit lengths mean the alignment shift cannot exceed 127 either way,
    // so neither side's shifted magnitude loses a bit.
    if shift >= 0 {
        (left.magnitude << shift as u32).cmp(&right.magnitude)
    } else {
        left.magnitude.cmp(&(right.magnitude << (-shift) as u32))
    }
}

/// Decomposes a JSON number without rounding: `serde_json` keeps integers and
/// floats apart, and both are representable as `sign · magnitude · 2^exponent`.
fn exact(number: &Number) -> Exact {
    if let Some(value) = number.as_u64() {
        return Exact {
            negative: false,
            magnitude: u128::from(value),
            exponent: 0,
        };
    }

    if let Some(value) = number.as_i64() {
        return Exact {
            negative: true,
            magnitude: u128::from(value.unsigned_abs()),
            exponent: 0,
        };
    }

    let value = number
        .as_f64()
        .expect("a JSON number is an integer or a finite float");
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let fraction = u128::from(bits & ((1u64 << 52) - 1));

    let (magnitude, exponent) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1u128 << 52), biased - 1075)
    };

    Exact {
        negative: bits >> 63 == 1,
        magnitude,
        exponent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    fn integer(value: i64) -> Number {
        Number::from(value)
    }

    fn natural(value: u64) -> Number {
        Number::from(value)
    }

    fn float(value: f64) -> Number {
        Number::from_f64(value).expect("the corpus holds only finite floats")
    }

    /// The values DEC-006 names: ±2^53, i64/u64 extremes, subnormals, signed
    /// zeros and large integral floats.
    fn corpus() -> Vec<Number> {
        vec![
            integer(i64::MIN),
            integer(-1),
            integer(0),
            integer(1),
            integer(1 << 53),
            integer((1 << 53) + 1),
            natural(0),
            natural(1 << 53),
            natural((1 << 53) + 1),
            natural(u64::MAX),
            float(-0.0),
            float(0.0),
            float(-5e-324),
            float(5e-324),
            float(1e-323),
            float(f64::MIN_POSITIVE),
            float(1.0),
            float(9007199254740992.0),
            float(-(2f64.powi(63))),
            float(1e308),
            float(-1e308),
            float(f64::MAX),
            float(f64::MIN),
        ]
    }

    #[test]
    fn the_same_value_in_either_representation_compares_equal() {
        let cases = [
            (integer(1), float(1.0)),
            (integer(0), float(0.0)),
            (integer(0), float(-0.0)),
            (natural(1 << 53), float(9007199254740992.0)),
            (integer(i64::MIN), float(-(2f64.powi(63)))),
            (integer(-2), float(-2.0)),
        ];

        for (left, right) in cases {
            assert_eq!(compare(&left, &right), Ordering::Equal, "{left} vs {right}");
            assert_eq!(compare(&right, &left), Ordering::Equal, "{right} vs {left}");
        }
    }

    #[test]
    fn integers_beyond_the_float_mantissa_compare_exactly() {
        let below = natural(9007199254740992);
        let above = natural(9007199254740993);
        let rounded = float(9007199254740992.0);

        assert_eq!(compare(&below, &above), Ordering::Less);
        assert_eq!(compare(&below, &rounded), Ordering::Equal);
        assert_eq!(compare(&above, &rounded), Ordering::Greater);
    }

    #[test]
    fn the_domain_switching_cycle_does_not_arise() {
        // DEC-006's counterexample. Per-pair domain switching yields z < a,
        // a < m and m < z; exact comparison makes m and z the same value.
        let z = natural(9007199254740992);
        let a = natural(9007199254740993);
        let m = float(9007199254740992.0);

        assert_eq!(compare(&z, &a), Ordering::Less);
        assert_eq!(compare(&a, &m), Ordering::Greater);
        assert_eq!(compare(&m, &z), Ordering::Equal);
        assert_ne!(
            compare(&m, &z),
            Ordering::Less,
            "that would close the cycle"
        );
    }

    #[test]
    fn subnormals_order_by_the_value_they_represent() {
        let smallest = float(5e-324);
        let twice = float(1e-323);

        assert_eq!(compare(&smallest, &twice), Ordering::Less);
        assert_eq!(compare(&float(0.0), &smallest), Ordering::Less);
        assert_eq!(compare(&integer(0), &smallest), Ordering::Less);
        assert_eq!(compare(&float(-5e-324), &float(0.0)), Ordering::Less);
        assert_eq!(compare(&float(-5e-324), &smallest), Ordering::Less);
    }

    #[test]
    fn signed_zeros_are_one_value_for_ordering() {
        assert_eq!(compare(&float(0.0), &float(-0.0)), Ordering::Equal);
        assert_eq!(compare(&float(-0.0), &integer(0)), Ordering::Equal);
        assert_eq!(compare(&float(-0.0), &integer(1)), Ordering::Less);
        assert_eq!(compare(&integer(-1), &float(-0.0)), Ordering::Less);
    }

    #[test]
    fn the_extremes_each_representation_can_hold_order_correctly() {
        assert_eq!(compare(&integer(i64::MIN), &integer(-1)), Ordering::Less);
        assert_eq!(compare(&integer(-1), &integer(0)), Ordering::Less);
        assert_eq!(compare(&natural(0), &natural(u64::MAX)), Ordering::Less);
        assert_eq!(
            compare(&natural(u64::MAX), &float(f64::MAX)),
            Ordering::Less
        );
        assert_eq!(
            compare(&float(f64::MIN), &integer(i64::MIN)),
            Ordering::Less
        );
        assert_eq!(
            compare(&float(1e308), &natural(u64::MAX)),
            Ordering::Greater
        );
    }

    fn numbers() -> impl Strategy<Value = Number> {
        prop_oneof![
            3 => any::<i64>().prop_map(Number::from),
            3 => any::<u64>().prop_map(Number::from),
            4 => any::<u64>()
                .prop_map(f64::from_bits)
                .prop_filter("a JSON number is finite", |value| value.is_finite())
                .prop_map(|value| Number::from_f64(value).expect("filtered to finite")),
            2 => prop::sample::select(corpus()),
        ]
    }

    proptest! {
        /// Total: every pair is ordered one way or the other, and the two
        /// directions agree.
        #[test]
        fn compare_is_total(left in numbers(), right in numbers()) {
            let forward = compare(&left, &right);
            let backward = compare(&right, &left);

            prop_assert_eq!(forward, backward.reverse(), "{} vs {}", left, right);
        }

        /// Transitive: sorting by the comparator yields a chain in which no
        /// pair is out of order — a comparator that cycles cannot survive this,
        /// because no arrangement of a cycle is consistent.
        #[test]
        fn compare_is_transitive(first in numbers(), second in numbers(), third in numbers()) {
            let mut values = [first, second, third];
            values.sort_by(|left, right| compare(left, right));
            let [smallest, middle, largest] = &values;

            let smallest_middle = compare(smallest, middle);
            let middle_largest = compare(middle, largest);

            prop_assert_ne!(smallest_middle, Ordering::Greater, "{:?} vs {:?}", smallest, middle);
            prop_assert_ne!(middle_largest, Ordering::Greater, "{:?} vs {:?}", middle, largest);
            prop_assert_ne!(compare(smallest, largest), Ordering::Greater, "{:?} vs {:?}", smallest, largest);

            if smallest_middle == Ordering::Less && middle_largest == Ordering::Less {
                prop_assert_eq!(compare(smallest, largest), Ordering::Less, "{:?} < {:?} < {:?}", smallest, middle, largest);
            }
        }

        /// Structural equality implies ordering equality: the ordering never
        /// contradicts `serde_json`'s own equality for one spelling.
        #[test]
        fn equal_numbers_compare_equal(left in numbers(), right in numbers()) {
            prop_assert!(
                left != right || compare(&left, &right) == Ordering::Equal,
                "{} and {} are equal but compared otherwise",
                left,
                right
            );
        }
    }
}
