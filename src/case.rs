//! Identifier case conversion used to derive default variant names.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Case {
	Snake,
	Kebab,
	ScreamingSnake,
	ScreamingKebab,
	Lower,
	Upper,
	Camel,
	Pascal,
}

impl Case {
	pub const EXPECTED: &'static str = "snake_case, kebab-case, SCREAMING_SNAKE_CASE, SCREAMING-KEBAB-CASE, lowercase, UPPERCASE, camelCase or PascalCase";

	pub fn parse(s: &str) -> Option<Self> {
		Some(match s {
			"snake_case" => Self::Snake,
			"kebab-case" => Self::Kebab,
			"SCREAMING_SNAKE_CASE" => Self::ScreamingSnake,
			"SCREAMING-KEBAB-CASE" => Self::ScreamingKebab,
			"lowercase" => Self::Lower,
			"UPPERCASE" => Self::Upper,
			"camelCase" => Self::Camel,
			"PascalCase" => Self::Pascal,
			_ => return None,
		})
	}

	pub fn apply(self, ident: &str) -> String {
		let words = split_words(ident);
		match self {
			Self::Snake => words.join("_"),
			Self::Kebab => words.join("-"),
			Self::ScreamingSnake => words.join("_").to_uppercase(),
			Self::ScreamingKebab => words.join("-").to_uppercase(),
			Self::Lower => words.concat(),
			Self::Upper => words.concat().to_uppercase(),
			Self::Pascal => words.iter().map(|w| capitalize(w)).collect(),
			Self::Camel => words.iter().enumerate().map(|(i, w)| if i == 0 { w.clone() } else { capitalize(w) }).collect(),
		}
	}
}

fn capitalize(word: &str) -> String {
	let mut chars = word.chars();
	chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

/// Splits on `_`, lower->upper transitions, digit->upper transitions and the end of acronyms
/// (`HTTPServer` -> `http`, `server`). Words are lowercased.
fn split_words(s: &str) -> Vec<String> {
	let chars: Vec<char> = s.chars().collect();
	let mut words: Vec<String> = Vec::new();
	let mut current = String::new();
	for (i, &c) in chars.iter().enumerate() {
		if c == '_' {
			if !current.is_empty() {
				words.push(std::mem::take(&mut current));
			}
			continue;
		}
		if c.is_uppercase() && i > 0 {
			let prev = chars[i - 1];
			let after_lower_or_digit = prev.is_lowercase() || prev.is_ascii_digit();
			let acronym_end = prev.is_uppercase() && chars.get(i + 1).is_some_and(|n| n.is_lowercase());
			if (after_lower_or_digit || acronym_end) && !current.is_empty() {
				words.push(std::mem::take(&mut current));
			}
		}
		current.extend(c.to_lowercase());
	}
	if !current.is_empty() {
		words.push(current);
	}
	words
}

// White-box: `split_words` and `Case::apply` are private to the proc-macro crate and pure,
// so they are tested here at the boundaries; the derived behavior is covered in `tests/`.
#[cfg(test)]
mod tests {
	use super::*;

	fn words(s: &str) -> Vec<String> {
		split_words(s)
	}

	#[test]
	fn splits_plain_pascal_case() {
		assert_eq!(words("EnemyHpLt"), ["enemy", "hp", "lt"]);
		assert_eq!(words("Always"), ["always"]);
	}

	#[test]
	fn splits_acronyms() {
		assert_eq!(words("HTTPServer"), ["http", "server"]);
		assert_eq!(words("XMLHttpRequest"), ["xml", "http", "request"]);
		assert_eq!(words("ABC"), ["abc"]);
		assert_eq!(words("ParseHTTP"), ["parse", "http"]);
	}

	#[test]
	fn splits_after_digits_but_not_before_them() {
		assert_eq!(words("Hp2Pct"), ["hp2", "pct"]);
		assert_eq!(words("V10"), ["v10"]);
		assert_eq!(words("A1B2"), ["a1", "b2"]);
	}

	#[test]
	fn underscores_are_separators_and_never_produce_empty_words() {
		assert_eq!(words("Snake_Case"), ["snake", "case"]);
		assert_eq!(words("__a__b__"), ["a", "b"]);
		assert_eq!(words("_"), Vec::<String>::new());
		assert_eq!(words(""), Vec::<String>::new());
	}

	#[test]
	fn single_characters_and_lowercase_input() {
		assert_eq!(words("X"), ["x"]);
		assert_eq!(words("x"), ["x"]);
		assert_eq!(words("already_snake"), ["already", "snake"]);
		assert_eq!(words("lowercase"), ["lowercase"]);
	}

	#[test]
	fn non_ascii_letters_follow_unicode_case_rules() {
		assert_eq!(words("ÉcoleNormale"), ["école", "normale"]);
	}

	#[test]
	fn every_case_has_the_expected_shape() {
		let expected = [
			(Case::Snake, "xml_http_request"),
			(Case::Kebab, "xml-http-request"),
			(Case::ScreamingSnake, "XML_HTTP_REQUEST"),
			(Case::ScreamingKebab, "XML-HTTP-REQUEST"),
			(Case::Lower, "xmlhttprequest"),
			(Case::Upper, "XMLHTTPREQUEST"),
			(Case::Camel, "xmlHttpRequest"),
			(Case::Pascal, "XmlHttpRequest"),
		];
		for (case, want) in expected {
			assert_eq!(case.apply("XMLHttpRequest"), want, "{case:?}");
		}
	}

	#[test]
	fn case_names_round_trip_through_parse() {
		let names = [
			("snake_case", Case::Snake),
			("kebab-case", Case::Kebab),
			("SCREAMING_SNAKE_CASE", Case::ScreamingSnake),
			("SCREAMING-KEBAB-CASE", Case::ScreamingKebab),
			("lowercase", Case::Lower),
			("UPPERCASE", Case::Upper),
			("camelCase", Case::Camel),
			("PascalCase", Case::Pascal),
		];
		for (name, case) in names {
			assert_eq!(Case::parse(name), Some(case));
		}
		for bad in ["", "Snake_Case", "snake", "kebab_case", "PASCALCASE", "snake_case "] {
			assert_eq!(Case::parse(bad), None, "{bad:?}");
		}
	}
}
