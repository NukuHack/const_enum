//! Contract of the full `ConstFromPrimitive` API:
//! * `to_primitive(v)` is the discriminant of `v`;
//! * `try_from_primitive(n)` is `Some(v)` iff `v` has discriminant `n`, otherwise `None`;
//! * `from_primitive_or(n, f)` is `try_from_primitive(n)` or `f`;
//! * `From`/`TryFrom` agree with the inherent functions, `TryFrom` returns the rejected number;
//! * all inherent functions are usable in `const` contexts.
#![allow(dead_code)]

use const_enum::ConstFromPrimitive;

const BASE: u8 = 100;

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
#[repr(u8)]
enum Op {
	Nop = 0,
	Add,
	Sub = 10,
	Mul,
	Big = BASE + 5,
}

const OP_DISCRIMINANTS: [(u8, Op); 5] = [(0, Op::Nop), (1, Op::Add), (10, Op::Sub), (11, Op::Mul), (105, Op::Big)];

#[test]
fn to_primitive_returns_each_discriminant() {
	for (n, op) in OP_DISCRIMINANTS {
		assert_eq!(op.to_primitive(), n);
	}
}

#[test]
fn try_from_primitive_is_exact_over_the_whole_domain() {
	for n in 0..=u8::MAX {
		let expected = OP_DISCRIMINANTS.iter().find(|(d, _)| *d == n).map(|(_, op)| *op);
		assert_eq!(Op::try_from_primitive(n), expected, "n = {n}");
	}
}

#[test]
fn implicit_discriminants_continue_from_the_previous_variant() {
	assert_eq!(Op::try_from_primitive(1), Some(Op::Add));
	assert_eq!(Op::try_from_primitive(11), Some(Op::Mul));
	assert_eq!(Op::try_from_primitive(2), None);
	assert_eq!(Op::try_from_primitive(12), None);
}

#[test]
fn discriminants_may_use_constants() {
	assert_eq!(Op::try_from_primitive(BASE + 5), Some(Op::Big));
	assert_eq!(Op::Big.to_primitive(), 105);
}

#[test]
fn from_primitive_or_returns_the_match_or_the_fallback() {
	assert_eq!(Op::from_primitive_or(10, Op::Nop), Op::Sub);
	assert_eq!(Op::from_primitive_or(7, Op::Mul), Op::Mul);
	assert_eq!(Op::from_primitive_or(u8::MAX, Op::Big), Op::Big);
	assert_eq!(Op::from_primitive_or(0, Op::Big), Op::Nop, "a match must win over the fallback");
}

#[test]
fn try_from_agrees_with_try_from_primitive() {
	for n in 0..=u8::MAX {
		match Op::try_from_primitive(n) {
			Some(op) => assert_eq!(Op::try_from(n), Ok(op)),
			None => assert_eq!(Op::try_from(n), Err(n)),
		}
	}
}

#[test]
fn from_enum_for_repr_agrees_with_to_primitive() {
	for (n, op) in OP_DISCRIMINANTS {
		assert_eq!(u8::from(op), n);
		let via_into: u8 = op.into();
		assert_eq!(via_into, n);
	}
}

#[test]
fn try_into_works_in_both_directions() {
	let op: Result<Op, _> = 10u8.try_into();
	assert_eq!(op, Ok(Op::Sub));
	let op: Result<Op, u8> = 3u8.try_into();
	assert_eq!(op, Err(3));
}

const C_SOME: Option<Op> = Op::try_from_primitive(105);
const C_NONE: Option<Op> = Op::try_from_primitive(104);
const C_OR: Op = Op::from_primitive_or(2, Op::Add);
const C_NUM: u8 = Op::Sub.to_primitive();
static S_TABLE: [u8; 2] = [Op::Add.to_primitive(), Op::Mul.to_primitive()];

#[test]
fn everything_is_const_evaluable() {
	assert_eq!(C_SOME, Some(Op::Big));
	assert_eq!(C_NONE, None);
	assert_eq!(C_OR, Op::Add);
	assert_eq!(C_NUM, 10);
	assert_eq!(S_TABLE, [1, 11]);
}

const fn const_roundtrip(n: u8) -> Option<u8> {
	match Op::try_from_primitive(n) {
		Some(op) => Some(op.to_primitive()),
		None => None,
	}
}

#[test]
fn usable_inside_const_fn_and_const_asserts() {
	const _: () = assert!(matches!(const_roundtrip(10), Some(10)));
	const _: () = assert!(const_roundtrip(2).is_none());
	assert_eq!(const_roundtrip(11), Some(11));
}

// Every integer repr, with its extremes as discriminants. `usize`/`isize` use the 32-bit range
// because larger discriminants are not portable to 32-bit targets.
macro_rules! repr_boundary_test {
	($test:ident, $enum:ident, $ty:ty, $min:expr, $max:expr) => {
		#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
		#[repr($ty)]
		enum $enum {
			Min = $min,
			Max = $max,
		}

		#[test]
		fn $test() {
			let (min, max): ($ty, $ty) = ($min, $max);
			assert_eq!($enum::try_from_primitive(min), Some($enum::Min));
			assert_eq!($enum::try_from_primitive(max), Some($enum::Max));
			assert_eq!($enum::try_from_primitive(min + 1), None);
			assert_eq!($enum::try_from_primitive(max - 1), None);
			assert_eq!($enum::Min.to_primitive(), min);
			assert_eq!($enum::Max.to_primitive(), max);
			assert_eq!(<$enum as TryFrom<$ty>>::try_from(max - 1), Err(max - 1));
			let one: $ty = 1;
			assert_eq!($enum::try_from_primitive(one), None);
		}
	};
}

repr_boundary_test!(repr_u8_boundaries, ReprU8, u8, u8::MIN, u8::MAX);
repr_boundary_test!(repr_u16_boundaries, ReprU16, u16, u16::MIN, u16::MAX);
repr_boundary_test!(repr_u32_boundaries, ReprU32, u32, u32::MIN, u32::MAX);
repr_boundary_test!(repr_u64_boundaries, ReprU64, u64, u64::MIN, u64::MAX);
repr_boundary_test!(repr_usize_boundaries, ReprUsize, usize, 0, u32::MAX as usize);
repr_boundary_test!(repr_i8_boundaries, ReprI8, i8, i8::MIN, i8::MAX);
repr_boundary_test!(repr_i16_boundaries, ReprI16, i16, i16::MIN, i16::MAX);
repr_boundary_test!(repr_i32_boundaries, ReprI32, i32, i32::MIN, i32::MAX);
repr_boundary_test!(repr_i64_boundaries, ReprI64, i64, i64::MIN, i64::MAX);
repr_boundary_test!(repr_isize_boundaries, ReprIsize, isize, i32::MIN as isize, i32::MAX as isize);

// 128-bit reprs are only stable on recent compilers, so they get their own tests.
repr_boundary_test!(repr_u128_boundaries, ReprU128, u128, u128::MIN, u128::MAX);
repr_boundary_test!(repr_i128_boundaries, ReprI128, i128, i128::MIN, i128::MAX);

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
enum NoRepr {
	Neg = -1,
	Pos = 5,
}

#[test]
fn enum_without_repr_uses_isize() {
	let n: isize = NoRepr::Neg.to_primitive();
	assert_eq!(n, -1);
	assert_eq!(NoRepr::try_from_primitive(5), Some(NoRepr::Pos));
	assert_eq!(NoRepr::try_from_primitive(isize::MAX), None);
	assert_eq!(isize::from(NoRepr::Pos), 5);
}

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
#[repr(align(4), u8)]
enum AlignBefore {
	A = 1,
	B,
}

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
#[repr(u16, align(8))]
enum AlignAfter {
	A = 1,
	B,
}

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
#[repr(align(2))]
#[repr(i16)]
enum ReprSplit {
	A = -2,
	B,
}

#[test]
fn repr_hints_next_to_the_integer_type_are_understood() {
	let a: u8 = AlignBefore::B.to_primitive();
	let b: u16 = AlignAfter::B.to_primitive();
	let c: i16 = ReprSplit::B.to_primitive();
	assert_eq!((a, b, c), (2, 2, -1));
	assert_eq!(ReprSplit::try_from_primitive(-2), Some(ReprSplit::A));
	assert_eq!(AlignBefore::try_from_primitive(2), Some(AlignBefore::B));
	assert_eq!(AlignAfter::try_from_primitive(3), None);
}

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
#[repr(i8)]
enum NegativeImplicit {
	A = -2,
	B,
	C,
	D,
}

#[test]
fn implicit_discriminants_count_up_through_zero() {
	assert_eq!(NegativeImplicit::try_from_primitive(-1), Some(NegativeImplicit::B));
	assert_eq!(NegativeImplicit::try_from_primitive(0), Some(NegativeImplicit::C));
	assert_eq!(NegativeImplicit::try_from_primitive(1), Some(NegativeImplicit::D));
	assert_eq!(NegativeImplicit::try_from_primitive(2), None);
}

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
#[repr(u8)]
enum PreludeNames {
	Some = 1,
	None = 2,
	Ok = 3,
	Err = 4,
	Option = 5,
	Result = 6,
	Self_ = 7,
}

#[test]
fn variants_named_like_prelude_items_do_not_confuse_the_expansion() {
	for n in 1..=7u8 {
		let v = PreludeNames::try_from_primitive(n).expect("variant exists");
		assert_eq!(v.to_primitive(), n);
	}
	assert_eq!(PreludeNames::try_from_primitive(0), None);
	assert_eq!(PreludeNames::try_from_primitive(8), None);
	assert_eq!(PreludeNames::try_from(1u8), Ok(PreludeNames::Some));
}

// Deliberately neither `Clone` nor `Copy`.
#[derive(Debug, PartialEq, ConstFromPrimitive)]
#[repr(u8)]
enum NotCopy {
	A = 1,
	B = 2,
}

#[test]
fn enums_that_are_not_copy_are_supported() {
	assert_eq!(NotCopy::try_from_primitive(1), Some(NotCopy::A));
	assert_eq!(NotCopy::from_primitive_or(9, NotCopy::B), NotCopy::B);
	assert_eq!(NotCopy::B.to_primitive(), 2);
	assert_eq!(u8::from(NotCopy::A), 1);
	assert_eq!(NotCopy::try_from(2u8), Ok(NotCopy::B));
}

#[test]
fn enums_declared_inside_a_function_are_supported() {
	#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
	#[repr(u8)]
	enum Local {
		A = 3,
		B,
	}

	assert_eq!(Local::try_from_primitive(4), Some(Local::B));
	assert_eq!(Local::A.to_primitive(), 3);
}

mod visibility {
	use const_enum::ConstFromPrimitive;

	/// Doc comments and foreign attributes on the enum and its variants must be ignored.
	#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
	#[repr(u8)]
	#[allow(dead_code)]
	pub(crate) enum Documented {
		/// First.
		#[allow(unused)]
		A = 1,
		/// Second.
		B,
	}

	#[test]
	fn doc_comments_and_foreign_attributes_are_ignored() {
		assert_eq!(Documented::try_from_primitive(2), Some(Documented::B));
		assert_eq!(Documented::A.to_primitive(), 1);
	}
}
