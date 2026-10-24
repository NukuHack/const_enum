//! Expansion of `#[derive(ConstFromPrimitive)]`: const conversions between a fieldless enum and its repr integer.

use crate::validate::{fieldless_enum, primitive_repr};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident};

pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
	let data = fieldless_enum(input, "ConstFromPrimitive")?;
	let name = &input.ident;
	let repr = primitive_repr(&input.attrs)?;

	let idents: Vec<&Ident> = data.variants.iter().map(|v| &v.ident).collect();
	// Index-based names cannot collide with variants, which are used as paths only.
	let consts: Vec<Ident> = (0..idents.len()).map(|i| format_ident!("__DISCR_{}", i)).collect();
	let mut defaults = data.variants.iter().filter(|v| v.attrs.iter().any(|a| a.path().is_ident("default")));
	let default = defaults.next().map(|v| &v.ident);
	if let Some(extra) = defaults.next() {
		return Err(syn::Error::new_spanned(extra, "only one variant can be marked `#[default]`"));
	}

	let from_primitive = default.map(|default| {
		quote! {
			/// Converts a primitive to the enum, returning the `#[default]` variant if no variant matches.
			#[inline]
			pub const fn from_primitive(number: #repr) -> Self {
				match Self::try_from_primitive(number) {
					::core::option::Option::Some(v) => v,
					::core::option::Option::None => Self::#default,
				}
			}
		}
	});

	Ok(quote! {
		impl #name {
			/// The discriminant of this variant.
			#[inline]
			pub const fn to_primitive(self) -> #repr {
				self as #repr
			}

			/// Converts a primitive to the enum, or `None` if no variant has that discriminant.
			pub const fn try_from_primitive(number: #repr) -> ::core::option::Option<Self> {
				#( const #consts: #repr = #name::#idents as #repr; )*
				match number {
					#( #consts => ::core::option::Option::Some(Self::#idents), )*
					_ => ::core::option::Option::None,
				}
			}

			/// Converts a primitive to the enum, using `fallback` if no variant matches.
			#[inline]
			pub const fn from_primitive_or(number: #repr, fallback: Self) -> Self {
				match Self::try_from_primitive(number) {
					::core::option::Option::Some(v) => v,
					::core::option::Option::None => fallback,
				}
			}

			#from_primitive
		}

		impl ::core::convert::From<#name> for #repr {
			#[inline]
			fn from(value: #name) -> Self {
				value.to_primitive()
			}
		}

		impl ::core::convert::TryFrom<#repr> for #name {
			/// The rejected number.
			type Error = #repr;

			#[inline]
			fn try_from(number: #repr) -> ::core::result::Result<Self, Self::Error> {
				match Self::try_from_primitive(number) {
					::core::option::Option::Some(v) => ::core::result::Result::Ok(v),
					::core::option::Option::None => ::core::result::Result::Err(number),
				}
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
		// Spaces differ between token printers; strip them to compare shapes.
		expand(&input).map(|t| t.to_string().replace(' ', ""))
	}

	#[test]
	fn rejects_invalid_targets_with_a_clear_message() {
		for (src, needle) in
			[("struct S;", "only supports enums"), ("enum E<T> { A }", "generic"), ("enum E {}", "at least one variant"), ("enum E { A(u8) }", "fieldless")]
		{
			let err = expand_src(src).expect_err(src);
			assert!(err.to_string().contains(needle), "{src}: {err}");
			assert!(err.to_string().contains("ConstFromPrimitive"), "{src}: {err}");
		}
	}

	#[test]
	fn from_primitive_is_generated_only_with_a_default_variant() {
		let without = expand_src("#[repr(u8)] enum E { A = 1 }").expect("valid");
		let with = expand_src("#[repr(u8)] enum E { #[default] A = 1 }").expect("valid");
		assert!(!without.contains("constfnfrom_primitive("));
		assert!(with.contains("constfnfrom_primitive("));
	}

	#[test]
	fn two_default_variants_are_rejected() {
		let err = expand_src("#[repr(u8)] enum E { #[default] A = 1, #[default] B = 2 }").expect_err("two defaults");
		assert!(err.to_string().contains("only one variant"));
	}

	#[test]
	fn default_variant_is_the_fallback() {
		let out = expand_src("#[repr(u8)] enum E { A = 1, #[default] B = 2 }").expect("valid");
		assert!(out.contains("None=>Self::B"));
	}

	#[test]
	fn uses_the_repr_type_in_every_signature() {
		let out = expand_src("#[repr(i16)] enum E { A }").expect("valid");
		assert!(out.contains("to_primitive(self)->i16"));
		assert!(out.contains("try_from_primitive(number:i16)"));
		assert!(out.contains("From<E>fori16"));
	}
}
