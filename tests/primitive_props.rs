//! Laws of `ConstFromPrimitive` over arbitrary inputs.
//! On failure proptest prints the minimal failing input and stores its seed in `proptest-regressions/`.
#![allow(dead_code)]

use const_enum::ConstFromPrimitive;
use proptest::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, ConstFromPrimitive)]
#[repr(i64)]
enum Sparse {
	#[default]
	Zero = 0,
	Min = i64::MIN,
	Max = i64::MAX,
	Neg = -9,
	Odd = 77,
}

const SPARSE_VALUES: [i64; 5] = [0, i64::MIN, i64::MAX, -9, 77];

#[derive(Clone, Copy, Debug, Default, PartialEq, ConstFromPrimitive)]
#[repr(u16)]
enum Narrow {
	#[default]
	A = 0,
	B = 255,
	C = 256,
	D = 65535,
}

const NARROW_VALUES: [u16; 4] = [0, 255, 256, 65535];

proptest! {
	#[test]
	fn try_from_primitive_succeeds_exactly_for_declared_discriminants(n in any::<i64>()) {
		prop_assert_eq!(Sparse::try_from_primitive(n).is_some(), SPARSE_VALUES.contains(&n));
	}

	#[test]
	fn successful_conversion_round_trips(n in any::<i64>()) {
		if let Some(v) = Sparse::try_from_primitive(n) {
			prop_assert_eq!(v.to_primitive(), n);
		}
	}

	#[test]
	fn from_primitive_is_total_and_falls_back_to_default(n in any::<i64>()) {
		let v = Sparse::from_primitive(n);
		if SPARSE_VALUES.contains(&n) {
			prop_assert_eq!(v.to_primitive(), n);
		} else {
			prop_assert_eq!(v, Sparse::Zero);
		}
	}

	#[test]
	fn from_primitive_or_returns_match_or_fallback(n in any::<i64>(), fallback_index in 0..SPARSE_VALUES.len()) {
		let fallback = Sparse::try_from_primitive(SPARSE_VALUES[fallback_index]).expect("declared value");
		let expected = Sparse::try_from_primitive(n).unwrap_or(fallback);
		prop_assert_eq!(Sparse::from_primitive_or(n, fallback), expected);
	}

	#[test]
	fn try_from_trait_agrees_with_the_inherent_function(n in any::<i64>()) {
		match Sparse::try_from_primitive(n) {
			Some(v) => prop_assert_eq!(Sparse::try_from(n), Ok(v)),
			None => prop_assert_eq!(Sparse::try_from(n), Err(n)),
		}
	}

	#[test]
	fn narrow_repr_laws_hold(n in any::<u16>()) {
		prop_assert_eq!(Narrow::try_from_primitive(n).is_some(), NARROW_VALUES.contains(&n));
		let v = Narrow::from_primitive(n);
		prop_assert_eq!(v.to_primitive() == n, NARROW_VALUES.contains(&n));
		prop_assert_eq!(u16::from(v), v.to_primitive());
	}
}

#[test]
fn narrow_repr_is_checked_exhaustively() {
	for n in 0..=u16::MAX {
		let expected = NARROW_VALUES.contains(&n);
		assert_eq!(Narrow::try_from_primitive(n).is_some(), expected, "n = {n}");
	}
}
