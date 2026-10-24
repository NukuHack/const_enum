//! Expansion of `#[derive(VariantName)]`: variant names, aliases, parsing and variant tables.

use crate::case::Case;
use crate::validate::{fieldless_enum, str_value};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::ext::IdentExt;
use syn::{DeriveInput, Ident, LitByteStr, LitStr};

pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
	let data = fieldless_enum(input, "VariantName")?;
	let type_name = &input.ident;

	let mut case = Case::Snake;
	for attr in input.attrs.iter().filter(|a| a.path().is_ident("name_case")) {
		let lit = str_value(attr, "name_case")?;
		case = Case::parse(&lit.value()).ok_or_else(|| syn::Error::new_spanned(&lit, format!("unknown case; expected {}", Case::EXPECTED)))?;
	}

	let mut idents: Vec<&Ident> = Vec::new();
	let mut names: Vec<LitStr> = Vec::new();
	let mut patterns: Vec<TokenStream> = Vec::new();
	let mut flat: Vec<(LitStr, &Ident)> = Vec::new();
	let mut seen: HashSet<String> = HashSet::new();

	for variant in &data.variants {
		let mut name = LitStr::new(&case.apply(&variant.ident.unraw().to_string()), variant.ident.span());
		let mut explicit = false;
		let mut aliases: Vec<LitStr> = Vec::new();
		for attr in &variant.attrs {
			if attr.path().is_ident("name") {
				if explicit {
					return Err(syn::Error::new_spanned(attr, "duplicate `#[name]`"));
				}
				name = str_value(attr, "name")?;
				explicit = true;
			} else if attr.path().is_ident("alias") {
				aliases.push(str_value(attr, "alias")?);
			}
		}

		for s in std::iter::once(&name).chain(aliases.iter()) {
			if !seen.insert(s.value()) {
				return Err(syn::Error::new_spanned(s, format!("duplicate name `{}`", s.value())));
			}
			flat.push((s.clone(), &variant.ident));
		}

		let to_bytes = |l: &LitStr| LitByteStr::new(l.value().as_bytes(), l.span());
		let (primary, rest) = (to_bytes(&name), aliases.iter().map(to_bytes));
		patterns.push(quote! { #primary #( | #rest )* });
		idents.push(&variant.ident);
		names.push(name);
	}

	let count = idents.len();
	let indices: Vec<usize> = (0..count).collect();
	let flat_names: Vec<&LitStr> = flat.iter().map(|(n, _)| n).collect();
	let flat_idents: Vec<&Ident> = flat.iter().map(|(_, i)| *i).collect();

	Ok(quote! {
		impl #type_name {
			/// Number of variants.
			pub const COUNT: usize = #count;

			/// Every variant, in declaration order.
			pub const ALL: [Self; #count] = [ #( Self::#idents ),* ];

			/// Every variant's primary name, in declaration order.
			pub const NAMES: [&'static str; #count] = [ #( #names ),* ];

			/// The primary name of this variant.
			pub const fn name(&self) -> &'static str {
				match *self {
					#( Self::#idents => #names, )*
				}
			}

			/// Position in declaration order (index into `ALL` / `NAMES`).
			pub const fn index(&self) -> usize {
				match *self {
					#( Self::#idents => #indices, )*
				}
			}

			/// Inverse of [`Self::index`].
			pub const fn from_index(index: usize) -> ::core::option::Option<Self> {
				match index {
					#( #indices => ::core::option::Option::Some(Self::#idents), )*
					_ => ::core::option::Option::None,
				}
			}

			/// Parses a primary name or alias (exact match).
			pub const fn parse(s: &str) -> ::core::option::Option<Self> {
				// `match` on `&str` is still unstable in `const fn`; byte-string patterns on the bytes are
				// stable and compile to a decision tree instead of a linear scan.
				match s.as_bytes() {
					#( #patterns => ::core::option::Option::Some(Self::#idents), )*
					_ => ::core::option::Option::None,
				}
			}

			/// Like [`Self::parse`] but ASCII case-insensitive.
			pub const fn parse_ignore_ascii_case(s: &str) -> ::core::option::Option<Self> {
				#( if Self::__name_eq_ignore_case(s, #flat_names) { return ::core::option::Option::Some(Self::#flat_idents); } )*
				::core::option::Option::None
			}

			#[doc(hidden)]
			#[inline]
			const fn __name_eq_ignore_case(a: &str, b: &str) -> bool {
				let (a, b) = (a.as_bytes(), b.as_bytes());
				if a.len() != b.len() {
					return false;
				}
				let mut i = 0;
				while i < a.len() {
					if !a[i].eq_ignore_ascii_case(&b[i]) {
						return false;
					}
					i += 1;
				}
				true
			}
		}

		impl ::core::fmt::Display for #type_name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
				f.write_str(self.name())
			}
		}
	})
}

// White-box: `expand` is private to the proc-macro crate; this is the only place its error paths
// can be asserted without a compile-fail harness.
#[cfg(test)]
mod tests {
	use super::*;

	fn expand_src(src: &str) -> syn::Result<String> {
		let input: DeriveInput = syn::parse_str(src).expect("test input must be valid Rust");
		expand(&input).map(|t| t.to_string())
	}

	fn error_of(src: &str) -> String {
		expand_src(src).expect_err(src).to_string()
	}

	#[test]
	fn rejects_invalid_targets_with_a_clear_message() {
		for (src, needle) in
			[("struct S;", "only supports enums"), ("enum E<T> { A }", "generic"), ("enum E {}", "at least one variant"), ("enum E { A(u8) }", "fieldless")]
		{
			let err = error_of(src);
			assert!(err.contains(needle), "{src}: {err}");
			assert!(err.contains("VariantName"), "{src}: {err}");
		}
	}

	#[test]
	fn duplicate_names_are_rejected_in_every_combination() {
		assert!(error_of("enum E { A, #[name = \"a\"] B }").contains("duplicate name `a`"));
		assert!(error_of("enum E { A, #[alias = \"a\"] B }").contains("duplicate name `a`"));
		assert!(error_of("enum E { #[alias = \"x\"] A, #[alias = \"x\"] B }").contains("duplicate name `x`"));
		assert!(error_of("enum E { #[alias = \"x\"] #[alias = \"x\"] A }").contains("duplicate name `x`"));
		assert!(error_of("enum E { #[alias = \"a\"] A }").contains("duplicate name `a`"));
	}

	#[test]
	fn default_names_that_collide_after_case_conversion_are_rejected() {
		assert!(error_of("enum E { FooBar, Foo_Bar }").contains("duplicate name `foo_bar`"));
		assert!(error_of("#[name_case = \"lowercase\"] enum E { FooBar, Foobar }").contains("duplicate name `foobar`"));
	}

	#[test]
	fn malformed_attributes_are_rejected() {
		assert!(error_of("enum E { #[name] A }").contains("expected `#[name"));
		assert!(error_of("enum E { #[name(\"x\")] A }").contains("expected `#[name"));
		assert!(error_of("enum E { #[name = 5] A }").contains("expected `#[name"));
		assert!(error_of("enum E { #[alias = 5] A }").contains("expected `#[alias"));
		assert!(error_of("enum E { #[name = \"a\"] #[name = \"b\"] A }").contains("duplicate `#[name]`"));
		assert!(error_of("#[name_case] enum E { A }").contains("expected `#[name_case"));
	}

	#[test]
	fn unknown_case_lists_the_valid_ones() {
		let err = error_of("#[name_case = \"Title Case\"] enum E { A }");
		assert!(err.contains("unknown case"));
		assert!(err.contains("kebab-case"));
	}

	#[test]
	fn valid_attributes_expand() {
		let out = expand_src("#[name_case = \"kebab-case\"] enum E { #[name = \"x\"] #[alias = \"y\"] A, B }").expect("valid");
		assert!(out.contains("b\"x\" | b\"y\""));
		assert!(out.contains("\"b\""));
	}
}
