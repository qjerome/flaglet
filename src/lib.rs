//! # flaglet 🏳️
//!
//! A proc-macro attribute that turns a plain Rust enum into a fully-featured
//! typed bitflag API — without a DSL, without a wrapper type you don't own,
//! and with first-class support for any derivable trait.
//!
//! ## Why flaglet?
//!
//! flaglet exists to stay out of your way — one attribute, and the rest is
//! plain Rust.
//!
//! - **Plain enum syntax.** Just annotate a regular enum — no custom DSL, no
//!   derive macro, no boilerplate. Your variants stay ordinary Rust.
//! - **A named companion type you own.** `#[flags]` on `Permissions` enum generates
//!   `PermissionsFlags`: a concrete struct in your crate that you can `impl`.
//! - **Derive forwarding.** Any `#[derive(...)]` placed on the enum is
//!   automatically forwarded to both the enum and the generated flags struct.
//! - **Auto-sized backing integer.** The macro picks the smallest integer type
//!   that fits your variants; override it with `#[flags(u64)]` when needed.
//! - **`no_std` out of the box.** No allocator required.
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! flaglet = "0.1"
//! ```
//!
//! ## Quickstart
//!
//! ```rust
//! use flaglet::flags;
//!
//! #[flags]
//! pub enum Permissions {
//!     Read    = 1 << 0,
//!     Write   = 1 << 1,
//!     Execute = 1 << 2,
//! }
//!
//! let mut perms = Permissions::flags(); // entry point on the enum itself
//! perms |= Permissions::Read;
//! perms |= Permissions::Write;
//!
//! assert!(perms.contains(Permissions::Read));
//! assert!(!perms.contains(Permissions::Execute));
//! ```
//!
//! ## Usage
//!
//! ### The `#[flags]` attribute
//!
//! Apply `#[flags]` to any enum whose variants are unit variants with explicit
//! power-of-2 discriminants. The macro generates a companion struct named
//! `{EnumName}Flags` in the same scope.
//!
//! ```rust
//! use flaglet::flags;
//! use core::mem::size_of;
//!
//! #[flags]
//! pub enum Permission {
//!     Read    = 0b001,
//!     Write   = 0b010,
//!     Execute = 0b100,
//! }
//! // Generates: pub struct PermissionFlags(u8);
//! assert!(PermissionFlags::empty().is_empty());
//! assert_eq!(size_of::<Permission>(), size_of::<u8>());
//! assert_eq!(size_of::<PermissionFlags>(), size_of::<u8>());
//! ```
//!
//! The backing integer type is chosen automatically from the largest
//! discriminant value (`u8`, `u16`, `u32`, or `u64`). You can override it
//! explicitly:
//!
//! ```rust
//! use flaglet::flags;
//! use core::mem::size_of;
//!
//! #[flags(u32)]
//! pub enum Permission {
//!     Read    = 0b001,
//!     Write   = 0b010,
//!     Execute = 0b100,
//! }
//! // Generates: pub struct PermissionFlags(u32);
//! assert!(PermissionFlags::empty().is_empty());
//! assert_eq!(size_of::<Permission>(), size_of::<u32>());
//! assert_eq!(size_of::<PermissionFlags>(), size_of::<u32>());
//! ```
//!
//! ### Constructing a flags value
//!
//! ```rust
//! use flaglet::flags;
//!
//! #[flags]
//! enum Permissions {
//!     Read    = 1 << 0,
//!     Write   = 1 << 1,
//!     Execute = 1 << 2,
//! }
//!
//! let _f = Permissions::flags();                           // empty
//! let _f = Permissions::all();                             // all variants set
//! let _f = Permissions::Read | Permissions::Write;         // combine variants
//! let _f = PermissionsFlags::from_flag(Permissions::Read); // single variant
//!
//! // Const context — use union() instead of |
//! const RW: PermissionsFlags = PermissionsFlags::empty()
//!     .union(Permissions::Read)
//!     .union(Permissions::Write);
//!
//! // From a raw integer (e.g. deserialization, FFI) — unknown bits are masked out
//! let _f = PermissionsFlags::from_bits(0b011_u8);
//! ```
//!
//! ### Setting and testing flags
//!
//! `set`, `unset`, `contains`, and `contains_any` accept both a single variant
//! and a combined flags value:
//!
//! ```rust
//! # use flaglet::flags;
//! # #[flags]
//! # enum Permissions {
//! #     Read    = 1 << 0,
//! #     Write   = 1 << 1,
//! #     Execute = 1 << 2,
//! # }
//! let mut f = PermissionsFlags::empty();
//! f.set(Permissions::Read);                        // set one flag
//! f.set(Permissions::Read | Permissions::Write);   // set multiple flags
//! f.unset(Permissions::Write);                     // clear one or more flags
//! let _ = f.is_empty();                            // true if no flags are set
//! let _ = f.bits();                                // raw integer value
//!
//! // contains: true if ALL specified bits are set
//! let _ = f.contains(Permissions::Read);
//! let _ = f.contains(Permissions::Read | Permissions::Write);
//!
//! // contains_any: true if AT LEAST ONE specified bit is set
//! let _ = f.contains_any(Permissions::Read | Permissions::Write);
//!
//! // is_disjoint: true if NO specified bits are set (opposite of contains_any)
//! let _ = f.is_disjoint(Permissions::Execute);
//! ```
//!
//! ### Bitwise operators
//!
//! All standard bitwise operators are implemented between the enum and the
//! flags struct, in both mutating and non-mutating forms:
//!
//! ```rust
//! # use flaglet::flags;
//! # #[flags]
//! # enum Permissions {
//! #     Read    = 1 << 0,
//! #     Write   = 1 << 1,
//! #     Execute = 1 << 2,
//! # }
//! // Enum OP Enum → PermissionsFlags (chains freely)
//! let mut f = Permissions::Read | Permissions::Write | Permissions::Execute;
//!
//! // Mutating
//! f |= Permissions::Read;    // set
//! f &= Permissions::Write;   // mask
//! f ^= Permissions::Execute; // toggle
//!
//! // Non-mutating (returns a new PermissionsFlags)
//! let g = f | Permissions::Read;
//! let h = f & Permissions::Write;
//! let i = f ^ Permissions::Execute;
//! let j = !f; // invert all bits
//!
//! // Flags OP Flags
//! let _k = f | g;
//! let _l = f & g;
//! ```
//!
//! ### Equality
//!
//! Cross-type `PartialEq` is implemented in both directions:
//!
//! ```rust
//! # use flaglet::flags;
//! # #[flags]
//! # enum Permissions {
//! #     Read    = 1 << 0,
//! #     Write   = 1 << 1,
//! #     Execute = 1 << 2,
//! # }
//! let mut f = PermissionsFlags::empty();
//! f.set(Permissions::Read);
//! assert!(f == Permissions::Read);   // PermissionsFlags == Permissions
//! assert!(Permissions::Read == f);   // Permissions == PermissionsFlags
//! ```
//!
//! Equality holds only when the flags value has exactly one bit set matching
//! the variant.
//!
//! ### Derive forwarding
//!
//! Any `#[derive(...)]` on the enum is forwarded to **both** the enum and the
//! generated flags struct. This includes third-party derives like
//! `serde::Serialize` or `rkyv::Archive` — no stubs, no wrappers:
//!
//! ```rust,ignore
//! #[flags]
//! #[derive(Debug, Serialize, Deserialize, Archive)]
//! pub enum Permissions {
//!     Read    = 1 << 0,
//!     Write   = 1 << 1,
//!     Execute = 1 << 2,
//! }
//! // Both Permissions and PermissionsFlags implement Debug, Serialize,
//! // Deserialize, and Archive.
//! ```
//!
//! ### Extending the generated type
//!
//! Because `PermissionsFlags` is a real struct generated in your crate's
//! scope, you can add your own methods to it directly:
//!
//! ```rust
//! # use flaglet::flags;
//! # #[flags]
//! # enum Permissions {
//! #     Read    = 1 << 0,
//! #     Write   = 1 << 1,
//! #     Execute = 1 << 2,
//! # }
//! impl PermissionsFlags {
//!     pub fn read_write() -> Self {
//!         Permissions::flags() | Permissions::Read | Permissions::Write
//!     }
//!
//!     pub fn is_read_only(&self) -> bool {
//!         self.contains(Permissions::Read) && !self.contains(Permissions::Write)
//!     }
//! }
//! ```
//!
//! ### Visibility
//!
//! The visibility of the enum is forwarded to both the enum and the generated
//! flags struct:
//!
//! ```rust
//! use flaglet::flags;
//!
//! #[flags] pub enum Foo { A = 1 }        // pub enum Foo + pub struct FooFlags
//! #[flags] pub(crate) enum Bar { A = 1 } // pub(crate) for both
//! #[flags] enum Baz { A = 1 }            // private for both
//! ```
//!
//! ### `no_std`
//!
//! `flaglet` is fully `no_std` compatible. All generated code uses `core::ops`
//! exclusively. No feature flag required.
//!
//! ## Compile-time validation
//!
//! The macro rejects invalid inputs with precise error messages pointing at
//! the offending token:
//!
//! - Variants with fields (tuple or struct variants) are rejected.
//! - Variants without an explicit discriminant are rejected.
//! - Discriminants that are zero or not a power of 2 are rejected.
//! - Discriminants that exceed `u64::MAX` are rejected.
//! - Discriminant expressions must be integer literals or `1 << N` form.
//!
//! ## License
//!
//! Licensed under either of [GPL-3.0-or-later](https://choosealicense.com/licenses/gpl-3.0/)
//! or [BSD-2-Clause](https://choosealicense.com/licenses/bsd-2-clause/) at your option.

use core::cmp::max;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Turns an enum into a typed bitflag API.
///
/// Generates a companion struct `{EnumName}Flags` with the full set of bitwise
/// operations, constructors, and testing methods. See the [crate-level
/// documentation](self) for the complete reference.
///
/// # Usage
///
/// ```rust
/// use flaglet::flags;
///
/// #[flags]
/// pub enum Permissions {
///     Read    = 1 << 0,
///     Write   = 1 << 1,
///     Execute = 1 << 2,
/// }
///
/// let f = Permissions::Read | Permissions::Write;
/// assert!(f.contains(Permissions::Read));
/// assert!(f.is_disjoint(Permissions::Execute));
/// ```
///
/// An explicit backing integer type can be provided as an argument:
///
/// ```rust
/// use flaglet::flags;
///
/// #[flags(u32)]
/// pub enum Permissions {
///     Read    = 1 << 0,
///     Write   = 1 << 1,
///     Execute = 1 << 2,
/// }
/// ```
#[proc_macro_attribute]
pub fn flags(attr: TokenStream, item: TokenStream) -> TokenStream {
    let override_ty: Option<syn::Type> = if attr.is_empty() {
        None
    } else {
        match syn::parse(attr) {
            Ok(ty) => Some(ty),
            Err(e) => return e.to_compile_error().into(),
        }
    };

    let input = parse_macro_input!(item as DeriveInput);

    match expand(input, override_ty) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn eval_expr(expr: &syn::Expr) -> syn::Result<u64> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit),
            ..
        }) => lit.base10_parse::<u64>().map_err(|_| {
            syn::Error::new_spanned(lit, "discriminant value is out of range for u64")
        }),
        syn::Expr::Binary(syn::ExprBinary {
            left,
            op: syn::BinOp::Shl(_),
            right,
            ..
        }) => {
            let base = eval_expr(left)?;
            let shift = eval_expr(right)?;
            if shift >= u64::BITS as u64 {
                return Err(syn::Error::new_spanned(
                    right,
                    "shift amount exceeds u64 bit width",
                ));
            }
            base.checked_shl(shift as u32)
                .ok_or_else(|| syn::Error::new_spanned(expr, "shift overflows u64"))
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "discriminant must be an integer literal or a `1 << N` expression",
        )),
    }
}

fn expand(input: DeriveInput, override_ty: Option<syn::Type>) -> syn::Result<TokenStream2> {
    let Data::Enum(ref data_enum) = input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "#[flags] can only be applied to enums",
        ));
    };

    let vis = &input.vis;
    let flags_enum = &input.ident;
    let flags_struct = quote::format_ident!("{}Flags", flags_enum);
    let attrs: Vec<_> = input.attrs.iter().collect();
    let mut variants = Vec::new();

    let mut max_val = 0;
    let mut all = 0;

    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "all variants must be unit variants (no fields allowed)",
            ));
        }

        let ident = &variant.ident;
        let Some((_, expr)) = &variant.discriminant else {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "all variants must have an explicit discriminant value (e.g. `Foo = 1`)",
            ));
        };

        let flag_value = eval_expr(expr)?;

        if flag_value == 0 || !flag_value.is_power_of_two() {
            return Err(syn::Error::new_spanned(
                expr,
                "discriminant must be a non-zero power of 2 (e.g. `1 << 3` or `0b0100`)",
            ));
        }

        max_val = max(flag_value, max_val);
        all |= flag_value;
        variants.push(quote! { #ident = #expr });
    }

    if max_val == 0 {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "flag discriminant values must be non-zero powers of 2",
        ));
    }

    let mut variant_idents = data_enum.variants.iter().map(|v| v.ident.to_string());
    let ex_v0 = variant_idents.next().unwrap_or_else(|| "A".to_string());
    let ex_v1 = variant_idents.next().unwrap_or_else(|| ex_v0.clone());

    let ty = if let Some(t) = override_ty {
        quote! { #t }
    } else {
        match max_val.ilog2() {
            0..8 => quote! { u8  },
            8..16 => quote! { u16 },
            16..32 => quote! { u32 },
            32..64 => quote! { u64 },
            _ => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "flag values exceed u64; maximum supported bit width is 64",
                ));
            }
        }
    };

    let doc_enum = format!(
        "Bitflag variants for [`{flags_struct}`].\n\n\
         Each variant represents a single power-of-two bit. Combine variants with `|` \
         to get a [`{flags_struct}`], or use [`{flags_enum}::flags()`] (empty) and \
         [`{flags_enum}::all()`] (every bit set) as starting points.\n\n\
         # Examples\n\n\
         ```rust,ignore\n\
         let f = {flags_enum}::{v0} | {flags_enum}::{v1};\n\
         assert!(f.contains({flags_enum}::{v0}));\n\n\
         let all = {flags_enum}::all();\n\
         assert!(all.contains({flags_enum}::{v0}));\n\
         ```",
        flags_struct = flags_struct,
        flags_enum = flags_enum,
        v0 = ex_v0,
        v1 = ex_v1,
    );
    let doc_flags_method = format!(
        "Returns an empty [`{flags_struct}`] with no bits set.\n\n\
         Shorthand for [`{flags_struct}::empty()`].",
        flags_struct = flags_struct,
    );
    let doc_all_method = format!(
        "Returns a [`{flags_struct}`] with every variant's bit set.\n\n\
         Shorthand for [`{flags_struct}::all()`].",
        flags_struct = flags_struct,
    );
    let doc_union_method = format!(
        "Returns a [`{flags_struct}`] with `self` and `flag` both set.\n\n\
         Const-compatible way to combine two variants without the `|` operator.",
        flags_struct = flags_struct,
    );
    let doc_flags_struct = format!(
        "A set of [`{flags_enum}`] flags backed by a `{ty}` integer.\n\n\
         Construct with [`{flags_struct}::empty()`], [`{flags_struct}::all()`], \
         [`{flags_struct}::from_flag()`], [`{flags_struct}::from_bits()`], \
         or the `|` operator on [`{flags_enum}`] variants. \
         Unknown bits are always masked out.\n\n\
         # Examples\n\n\
         ```rust,ignore\n\
         // Build from variants\n\
         let f = {flags_enum}::{v0} | {flags_enum}::{v1};\n\
         let f = {flags_struct}::from_flag({flags_enum}::{v0});\n\n\
         // Test\n\
         assert!(f.contains({flags_enum}::{v0}));\n\
         assert!(f.contains({flags_enum}::{v0} | {flags_enum}::{v1}));\n\
         assert!(f.contains_any({flags_enum}::{v0} | {flags_enum}::{v1}));\n\
         assert!(!f.is_empty());\n\
         ```",
        flags_enum = flags_enum,
        flags_struct = flags_struct,
        ty = ty.to_string().trim(),
        v0 = ex_v0,
        v1 = ex_v1,
    );

    Ok(quote! {
        #[doc = #doc_enum]
        #[repr(#ty)]
        #[derive(Clone, Copy)]
        #(#attrs)*
        #vis enum #flags_enum {
            #(#variants,)*
        }

        impl #flags_enum {
            #[doc = #doc_flags_method]
            #[inline]
            #vis const fn flags() -> #flags_struct {
                #flags_struct::empty()
            }

            #[doc = #doc_all_method]
            #[inline]
            #vis const fn all() -> #flags_struct {
                #flags_struct::all()
            }

            #[doc = #doc_union_method]
            #[inline]
            #vis const fn union(self, flag: Self) -> #flags_struct {
                #flags_struct(self as #ty | flag as #ty)
            }
        }

        impl core::ops::BitOr for #flags_enum {
            type Output = #flags_struct;
            fn bitor(self, rhs: Self) -> #flags_struct {
                #flags_struct(self as #ty | rhs as #ty)
            }
        }

        impl core::ops::BitAnd for #flags_enum {
            type Output = #flags_struct;
            fn bitand(self, rhs: Self) -> #flags_struct {
                #flags_struct(self as #ty & rhs as #ty)
            }
        }

        impl core::ops::BitXor for #flags_enum {
            type Output = #flags_struct;
            fn bitxor(self, rhs: Self) -> #flags_struct {
                #flags_struct(self as #ty ^ rhs as #ty)
            }
        }

        impl From<#flags_enum> for #ty {
            fn from(value: #flags_enum) -> #ty {
                value as #ty
            }
        }

        #[doc = #doc_flags_struct]
        #[derive(Clone, Copy)]
        #(#attrs)*
        #vis struct #flags_struct(#ty);

        impl #flags_struct {
            /// Creates a flags value with no bits set.
            #[inline]
            #vis const fn empty() -> Self {
                Self(0)
            }

            /// Creates a flags value with every variant's bit set.
            #[inline]
            #vis const fn all() -> Self {
                Self(#all as #ty)
            }

            /// Constructs a flags value with exactly one bit set.
            #[inline]
            #vis const fn from_flag(value: #flags_enum) -> Self {
                Self(value as #ty)
            }

            /// Constructs a flags value from a raw bit pattern.
            ///
            /// Bits that don't correspond to any variant are cleared.
            #[inline]
            #vis const fn from_bits(value: #ty) -> Self {
                Self(value & #all as #ty)
            }

            /// Returns the raw integer backing this flags value.
            #[inline]
            #vis const fn bits(&self) -> #ty {
                self.0
            }

            /// Returns a new flags value with `flag` added.
            ///
            /// Const-compatible alternative to the `|` operator.
            #[inline]
            #vis const fn union(self, flag: #flags_enum) -> Self {
                Self(self.0 | flag as #ty)
            }

            /// Returns `true` if no bits are set.
            #[inline]
            #vis const fn is_empty(&self) -> bool {
                self.0 == 0
            }

            /// Returns `true` if all bits are set.
            #[inline]
            #vis const fn is_all(&self) -> bool {
                self.0 == #all as #ty
            }

            /// Clears all flags.
            #[inline]
            #vis const fn clear(&mut self) {
                self.0 = 0
            }

            /// Sets every bit in `flags`.
            #[inline]
            #vis fn set<T>(&mut self, flags: T)
                where T: Into<Self> + Copy
            {
                *self |= flags.into();
            }

            /// Clears every bit in `flags`.
            ///
            /// No-op for bits that are not already set.
            #[inline]
            #vis fn unset<T>(&mut self, flags: T)
                where T: Into<Self> + Copy
            {
                *self &= !flags.into();
            }

            /// Returns `true` if every bit in `flags` is set.
            #[inline]
            #vis fn contains<T>(&self, flags: T) -> bool
                where T: Into<Self> + Copy
            {
                let flags = flags.into();
                self.0 & flags.0 == flags.0
            }

            /// Returns `true` if at least one bit in `flags` is set.
            #[inline]
            #vis fn contains_any<T>(&self, flags: T) -> bool
                where T: Into<Self> + Copy
            {
                self.0 & flags.into().0 != 0
            }

            /// Returns `true` if none of the bits in `flags` are set.
            #[inline]
            #vis fn is_disjoint<T>(&self, flags: T) -> bool
                where T: Into<Self> + Copy
            {
                let flags = flags.into();
                self.0 & flags.0 == 0
            }
        }

        impl From<#flags_enum> for #flags_struct {
            fn from(value: #flags_enum) -> Self {
                Self::from_flag(value)
            }
        }

        impl From<#ty> for #flags_struct {
            fn from(value: #ty) -> Self {
                Self(value & #all as #ty)
            }
        }

        impl core::ops::BitOrAssign<#flags_enum> for #flags_struct {
            fn bitor_assign(&mut self, flag: #flags_enum) {
                self.0 |= flag as #ty;
            }
        }

        impl core::ops::BitOrAssign<#flags_struct> for #flags_struct {
            fn bitor_assign(&mut self, other: #flags_struct) {
                self.0 |= other.0;
            }
        }

        impl core::ops::BitAndAssign<#flags_enum> for #flags_struct {
            fn bitand_assign(&mut self, flag: #flags_enum) {
                self.0 &= flag as #ty;
            }
        }

        impl core::ops::BitAndAssign<#flags_struct> for #flags_struct {
            fn bitand_assign(&mut self, other: #flags_struct) {
                self.0 &= other.0;
            }
        }

        impl core::ops::BitXorAssign<#flags_enum> for #flags_struct {
            fn bitxor_assign(&mut self, flag: #flags_enum) {
                self.0 ^= flag as #ty;
            }
        }

        impl core::ops::BitXorAssign<#flags_struct> for #flags_struct {
            fn bitxor_assign(&mut self, other: #flags_struct) {
                self.0 ^= other.0;
            }
        }

        impl core::ops::Not for #flags_struct {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0 & #all as #ty)
            }
        }

        impl core::ops::BitOr<#flags_enum> for #flags_struct {
            type Output = Self;
            fn bitor(self, flag: #flags_enum) -> Self {
                Self(self.0 | flag as #ty)
            }
        }

        impl core::ops::BitOr<#flags_struct> for #flags_struct {
            type Output = Self;
            fn bitor(self, other: #flags_struct) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl core::ops::BitAnd<#flags_enum> for #flags_struct {
            type Output = Self;
            fn bitand(self, flag: #flags_enum) -> Self {
                Self(self.0 & flag as #ty)
            }
        }

        impl core::ops::BitAnd<#flags_struct> for #flags_struct {
            type Output = Self;
            fn bitand(self, other: #flags_struct) -> Self {
                Self(self.0 & other.0)
            }
        }

        impl core::ops::BitXor<#flags_enum> for #flags_struct {
            type Output = Self;
            fn bitxor(self, flag: #flags_enum) -> Self {
                Self(self.0 ^ flag as #ty)
            }
        }

        impl core::ops::BitXor<#flags_struct> for #flags_struct {
            type Output = Self;
            fn bitxor(self, other: #flags_struct) -> Self {
                Self(self.0 ^ other.0)
            }
        }

        impl PartialEq<#flags_enum> for #flags_struct {
            fn eq(&self, other: &#flags_enum) -> bool {
                self.0 == *other as #ty
            }
        }

        impl PartialEq<#flags_struct> for #flags_enum {
            fn eq(&self, other: &#flags_struct) -> bool {
                other.0 == *self as #ty
            }
        }
    })
}
