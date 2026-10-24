//! Input validation and attribute parsing shared by both derives.

use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Data, DataEnum, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitStr, Meta};

const INT_REPRS: [&str; 12] = ["u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize"];

pub fn fieldless_enum<'a>(input: &'a DeriveInput, derive: &str) -> syn::Result<&'a DataEnum> {
	let Data::Enum(data) = &input.data else {
		return Err(syn::Error::new_spanned(&input.ident, format!("{derive} only supports enums")));
	};
	if !input.generics.params.is_empty() {
		return Err(syn::Error::new_spanned(&input.generics, format!("{derive} does not support generic enums")));
	}
	if data.variants.is_empty() {
		return Err(syn::Error::new_spanned(&input.ident, format!("{derive} requires at least one variant")));
	}
	if let Some(variant) = data.variants.iter().find(|v| !matches!(v.fields, Fields::Unit)) {
		return Err(syn::Error::new_spanned(variant, format!("{derive} only supports fieldless (unit) variants")));
	}
	Ok(data)
}

/// Falls back to `isize`, the type rustc uses for discriminants without a `#[repr]`.
pub fn primitive_repr(attrs: &[Attribute]) -> syn::Result<Ident> {
	let mut found = None;
	for attr in attrs.iter().filter(|a| a.path().is_ident("repr")) {
		attr.parse_nested_meta(|meta| {
			if let Some(id) = meta.path.get_ident() {
				if INT_REPRS.iter().any(|r| id == *r) {
					found = Some(id.clone());
				}
			}
			// Hints like `align(8)` carry arguments that must be consumed to keep parsing.
			if meta.input.peek(syn::token::Paren) {
				let content;
				syn::parenthesized!(content in meta.input);
				let _: TokenStream = content.parse()?;
			}
			Ok(())
		})?;
	}
	Ok(found.unwrap_or_else(|| Ident::new("isize", Span::call_site())))
}

pub fn str_value(attr: &Attribute, expected: &str) -> syn::Result<LitStr> {
	if let Meta::NameValue(nv) = &attr.meta {
		// A literal passed through `macro_rules!` arrives wrapped in an invisible group.
		let mut value = &nv.value;
		while let Expr::Group(group) = value {
			value = &group.expr;
		}
		if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
			return Ok(s.clone());
		}
	}
	Err(syn::Error::new_spanned(attr, format!("expected `#[{expected} = \"...\"]`")))
}

// White-box: these helpers are private to the proc-macro crate and cannot be reached from `tests/`.
#[cfg(test)]
mod tests {
	use super::*;

	fn parse(src: &str) -> DeriveInput {
		syn::parse_str(src).expect("test input must be valid Rust")
	}

	fn repr_of(src: &str) -> String {
		primitive_repr(&parse(src).attrs).expect("repr must parse").to_string()
	}

	fn err_of(src: &str) -> String {
		fieldless_enum(&parse(src), "Test").err().expect("must be rejected").to_string()
	}

	#[test]
	fn repr_defaults_to_isize() {
		assert_eq!(repr_of("enum E { A }"), "isize");
		assert_eq!(repr_of("#[repr(C)] enum E { A }"), "isize");
	}

	#[test]
	fn repr_finds_every_integer_type() {
		for ty in INT_REPRS {
			assert_eq!(repr_of(&format!("#[repr({ty})] enum E {{ A }}")), ty);
		}
	}

	#[test]
	fn repr_is_found_among_other_hints_in_any_order() {
		assert_eq!(repr_of("#[repr(C, u8)] enum E { A }"), "u8");
		assert_eq!(repr_of("#[repr(u16, C)] enum E { A }"), "u16");
		assert_eq!(repr_of("#[repr(align(8), i64)] enum E { A }"), "i64");
		assert_eq!(repr_of("#[repr(i32, align(8))] enum E { A }"), "i32");
	}

	#[test]
	fn repr_is_found_across_attributes_and_ignores_foreign_ones() {
		assert_eq!(repr_of("#[repr(C)] #[repr(u32)] enum E { A }"), "u32");
		assert_eq!(repr_of("#[derive(Debug)] #[doc = \"x\"] #[repr(u64)] enum E { A }"), "u64");
	}

	#[test]
	fn non_enum_is_rejected() {
		assert!(err_of("struct S;").contains("only supports enums"));
		assert!(err_of("union U { a: u8 }").contains("only supports enums"));
	}

	#[test]
	fn generic_enum_is_rejected() {
		assert!(err_of("enum E<T> { A }").contains("generic"));
		assert!(err_of("enum E<'a> { A }").contains("generic"));
	}

	#[test]
	fn empty_enum_is_rejected() {
		assert!(err_of("enum E {}").contains("at least one variant"));
	}

	#[test]
	fn data_carrying_variants_are_rejected() {
		assert!(err_of("enum E { A, B(u8) }").contains("fieldless"));
		assert!(err_of("enum E { A, B { x: u8 } }").contains("fieldless"));
	}

	#[test]
	fn valid_enum_is_accepted() {
		assert!(fieldless_enum(&parse("enum E { A, B = 3 }"), "Test").is_ok());
	}

	#[test]
	fn str_value_accepts_plain_string_only() {
		let ok = parse("#[name = \"x\"] enum E { A }");
		assert_eq!(str_value(&ok.attrs[0], "name").expect("string literal").value(), "x");
		for bad in ["#[name = 5] enum E { A }", "#[name] enum E { A }", "#[name(x)] enum E { A }", "#[name = x] enum E { A }"] {
			let input = parse(bad);
			assert!(str_value(&input.attrs[0], "name").is_err(), "{bad}");
		}
	}
}
