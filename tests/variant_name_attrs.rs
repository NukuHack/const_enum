//! Contract of the `VariantName` attributes:
//! * `#[name = ".."]` replaces the generated name, `#[alias = ".."]` adds accepted spellings;
//! * aliases are accepted by `parse`/`parse_ignore_ascii_case` but never listed in `NAMES` or returned by `name()`;
//! * `#[name_case = ".."]` selects how identifiers are converted, for every supported case.
#![allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]

use const_enum::VariantName;

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum Renamed {
	#[name = "hp_special"]
	SomethingOdd,
	#[name = "UPPER"]
	Shouting,
	#[name = ""]
	Empty,
	#[name = "with space"]
	Spaced,
	#[name = "quote\"and\\slash"]
	Escaped,
	#[name = "ünï-çødé"]
	Unicode,
	#[name = "line\nbreak"]
	Newline,
	Plain,
}

#[test]
fn explicit_names_replace_the_generated_ones() {
	assert_eq!(Renamed::SomethingOdd.name(), "hp_special");
	assert_eq!(Renamed::Shouting.name(), "UPPER");
	assert_eq!(Renamed::Plain.name(), "plain");
	assert_eq!(Renamed::parse("something_odd"), None, "the generated name no longer exists");
}

#[test]
fn explicit_names_may_be_any_string() {
	let expected = ["hp_special", "UPPER", "", "with space", "quote\"and\\slash", "ünï-çødé", "line\nbreak", "plain"];
	assert_eq!(Renamed::NAMES, expected);
	for (variant, name) in Renamed::ALL.into_iter().zip(expected) {
		assert_eq!(variant.name(), name);
		assert_eq!(Renamed::parse(name), Some(variant), "{name:?}");
		assert_eq!(variant.to_string(), name);
	}
}

#[test]
fn the_empty_string_is_a_valid_name_and_only_matches_itself() {
	assert_eq!(Renamed::parse(""), Some(Renamed::Empty));
	assert_eq!(Renamed::parse_ignore_ascii_case(""), Some(Renamed::Empty));
	assert_eq!(Renamed::parse(" "), None);
}

#[test]
fn ignore_case_does_not_fold_non_ascii_letters() {
	assert_eq!(Renamed::parse_ignore_ascii_case("ünï-çødé"), Some(Renamed::Unicode));
	assert_eq!(Renamed::parse_ignore_ascii_case("ÜNÏ-ÇØDÉ"), None);
	assert_eq!(Renamed::parse_ignore_ascii_case("upper"), Some(Renamed::Shouting));
	assert_eq!(Renamed::parse("upper"), None);
}

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum Aliased {
	#[alias = "a"]
	#[alias = "first"]
	Alpha,
	#[name = "bee"]
	#[alias = "B"]
	Beta,
	Gamma,
}

#[test]
fn aliases_are_accepted_by_parse() {
	assert_eq!(Aliased::parse("alpha"), Some(Aliased::Alpha));
	assert_eq!(Aliased::parse("a"), Some(Aliased::Alpha));
	assert_eq!(Aliased::parse("first"), Some(Aliased::Alpha));
	assert_eq!(Aliased::parse("bee"), Some(Aliased::Beta));
	assert_eq!(Aliased::parse("B"), Some(Aliased::Beta));
	assert_eq!(Aliased::parse("gamma"), Some(Aliased::Gamma));
}

#[test]
fn aliases_do_not_leak_into_the_tables() {
	assert_eq!(Aliased::NAMES, ["alpha", "bee", "gamma"]);
	assert_eq!(Aliased::COUNT, 3);
	assert_eq!(Aliased::ALL.len(), 3);
	assert_eq!(Aliased::Alpha.name(), "alpha");
	assert_eq!(Aliased::Beta.to_string(), "bee");
}

#[test]
fn aliases_are_accepted_by_the_case_insensitive_parser_too() {
	assert_eq!(Aliased::parse_ignore_ascii_case("FIRST"), Some(Aliased::Alpha));
	assert_eq!(Aliased::parse_ignore_ascii_case("b"), Some(Aliased::Beta), "alias `B` matches `b`");
	assert_eq!(Aliased::parse_ignore_ascii_case("BEE"), Some(Aliased::Beta));
}

#[test]
fn a_replaced_generated_name_is_not_an_alias() {
	assert_eq!(Aliased::parse("beta"), None);
}

#[test]
fn unknown_spellings_are_rejected() {
	for bad in ["", "al", "alphaa", "firs", "A", "Bee", "gam"] {
		assert_eq!(Aliased::parse(bad), None, "{bad:?}");
	}
}

macro_rules! name_case_test {
	($test:ident, $enum:ident, $case:literal, [$($expected:literal),+ $(,)?]) => {
		#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
		#[name_case = $case]
		enum $enum {
			EnemyHpLt,
			HTTPServer,
			Hp2Pct,
			ABC,
			X,
			XMLHttpRequest,
			Snake_Case,
		}

		#[test]
		fn $test() {
			assert_eq!($enum::NAMES, [$($expected),+]);
			for v in $enum::ALL {
				assert_eq!($enum::parse(v.name()), Some(v));
			}
		}
	};
}

name_case_test!(snake_case, CaseSnake, "snake_case", ["enemy_hp_lt", "http_server", "hp2_pct", "abc", "x", "xml_http_request", "snake_case"]);
name_case_test!(kebab_case, CaseKebab, "kebab-case", ["enemy-hp-lt", "http-server", "hp2-pct", "abc", "x", "xml-http-request", "snake-case"]);
name_case_test!(
	screaming_snake_case,
	CaseScreamingSnake,
	"SCREAMING_SNAKE_CASE",
	["ENEMY_HP_LT", "HTTP_SERVER", "HP2_PCT", "ABC", "X", "XML_HTTP_REQUEST", "SNAKE_CASE"]
);
name_case_test!(
	screaming_kebab_case,
	CaseScreamingKebab,
	"SCREAMING-KEBAB-CASE",
	["ENEMY-HP-LT", "HTTP-SERVER", "HP2-PCT", "ABC", "X", "XML-HTTP-REQUEST", "SNAKE-CASE"]
);
name_case_test!(lowercase, CaseLower, "lowercase", ["enemyhplt", "httpserver", "hp2pct", "abc", "x", "xmlhttprequest", "snakecase"]);
name_case_test!(uppercase, CaseUpper, "UPPERCASE", ["ENEMYHPLT", "HTTPSERVER", "HP2PCT", "ABC", "X", "XMLHTTPREQUEST", "SNAKECASE"]);
name_case_test!(camel_case, CaseCamel, "camelCase", ["enemyHpLt", "httpServer", "hp2Pct", "abc", "x", "xmlHttpRequest", "snakeCase"]);
name_case_test!(pascal_case, CasePascal, "PascalCase", ["EnemyHpLt", "HttpServer", "Hp2Pct", "Abc", "X", "XmlHttpRequest", "SnakeCase"]);

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
#[name_case = "kebab-case"]
enum CaseWithOverrides {
	FooBar,
	#[name = "keep_this_exactly"]
	BazQux,
	#[alias = "Alias_Is_Kept_Verbatim"]
	QuuxCorge,
}

#[test]
fn name_case_applies_to_generated_names_only() {
	assert_eq!(CaseWithOverrides::NAMES, ["foo-bar", "keep_this_exactly", "quux-corge"]);
	assert_eq!(CaseWithOverrides::parse("Alias_Is_Kept_Verbatim"), Some(CaseWithOverrides::QuuxCorge));
	assert_eq!(CaseWithOverrides::parse("alias-is-kept-verbatim"), None);
}

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum RawIdents {
	r#Match,
	r#Type,
	r#Async,
}

#[test]
fn raw_identifiers_lose_their_prefix() {
	assert_eq!(RawIdents::NAMES, ["match", "type", "async"]);
	assert_eq!(RawIdents::parse("type"), Some(RawIdents::r#Type));
}

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum NonAscii {
	Größe,
	Ärger,
	日本語,
}

#[test]
fn non_ascii_identifiers_are_lowercased_by_unicode_rules() {
	assert_eq!(NonAscii::NAMES[0], "größe");
	assert_eq!(NonAscii::NAMES[1], "ärger");
	assert_eq!(NonAscii::NAMES[2], "日本語");
	assert_eq!(NonAscii::parse("日本語"), Some(NonAscii::日本語));
}

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum Digits {
	V1,
	V10,
	Hp2,
	Hp2Lt,
	A1B2,
	Top10List,
}

#[test]
fn digits_stay_attached_to_the_preceding_word() {
	assert_eq!(Digits::NAMES, ["v1", "v10", "hp2", "hp2_lt", "a1_b2", "top10_list"]);
}
