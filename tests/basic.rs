use const_enum::{ConstFromPrimitive, VariantName};

const BASE: u8 = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ConstFromPrimitive, VariantName)]
#[repr(u8)]
enum Op {
	#[default]
	Nop = 0,
	Add,
	Sub = 10,
	Mul,
	Big = BASE + 5,
}

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
#[name_case = "kebab-case"]
enum Pred {
	Always,
	EnemyHpLt,
	HTTPServer,
	Hp2Pct,
	#[name = "custom"]
	#[alias = "odd"]
	#[alias = "weird"]
	Something,
}

#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
enum NoRepr {
	A = -1,
	B = 5,
}

// ---- everything below is evaluated at compile time ----
const A: Op = Op::from_primitive(11);
const B: Op = Op::from_primitive(77);
const C: Option<Op> = Op::try_from_primitive(105);
const D: u8 = Op::Sub.to_primitive();
const E: Op = Op::from_primitive_or(3, Op::Mul);
const N: NoRepr = match NoRepr::try_from_primitive(-1) {
	Some(v) => v,
	None => NoRepr::B,
};
const P: Option<Pred> = Pred::parse("enemy-hp-lt");
const P2: Option<Pred> = Pred::parse_ignore_ascii_case("HTTP-SERVER");
const NAME: &str = Pred::Hp2Pct.name();
static TABLE: [&str; Pred::COUNT] = Pred::NAMES;

#[test]
fn primitive() {
	assert_eq!(A, Op::Mul);
	assert_eq!(B, Op::Nop);
	assert_eq!(C, Some(Op::Big));
	assert_eq!(D, 10);
	assert_eq!(E, Op::Mul);
	assert_eq!(N, NoRepr::A);
	assert_eq!(Op::try_from_primitive(2), None);
	assert_eq!(Op::try_from(1u8), Ok(Op::Add));
	assert_eq!(Op::try_from(2u8), Err(2));
	assert_eq!(u8::from(Op::Big), 105);
}

#[test]
fn names() {
	assert_eq!(P, Some(Pred::EnemyHpLt));
	assert_eq!(P2, Some(Pred::HTTPServer));
	assert_eq!(NAME, "hp2-pct");
	assert_eq!(TABLE, ["always", "enemy-hp-lt", "http-server", "hp2-pct", "custom"]);
	assert_eq!(Pred::parse("weird"), Some(Pred::Something));
	assert_eq!(Pred::parse("something"), None);
	assert_eq!(Pred::Something.to_string(), "custom");
	assert_eq!(Pred::from_index(2), Some(Pred::HTTPServer));
	assert_eq!(Pred::Hp2Pct.index(), 3);
	assert_eq!(Pred::ALL.len(), Pred::COUNT);
	for p in Pred::ALL {
		assert_eq!(Pred::parse(p.name()), Some(p));
	}
}
