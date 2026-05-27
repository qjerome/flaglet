[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/qjerome/flaglet/rust.yml?style=for-the-badge)](https://github.com/qjerome/flaglet/actions/workflows/rust.yml)
[![Crates.io Version](https://img.shields.io/crates/v/flaglet?style=for-the-badge)](https://crates.io/crates/flaglet)
[![docs.rs](https://img.shields.io/docsrs/flaglet?style=for-the-badge&logo=docs.rs&color=blue)](https://docs.rs/flaglet)
[![License](https://img.shields.io/crates/l/flaglet?style=for-the-badge&color=purple)](https://crates.io/crates/flaglet)

<!-- cargo-rdme start -->

# flaglet 🏳️

A proc-macro attribute that turns a plain Rust enum into a fully-featured
typed bitflag API — without a DSL, without a wrapper type you don't own,
and with first-class support for any derivable trait.

## Why flaglet?

flaglet exists to stay out of your way — one attribute, and the rest is
plain Rust.

- **Plain enum syntax.** Just annotate a regular enum — no custom DSL, no
  derive macro, no boilerplate. Your variants stay ordinary Rust.
- **A named companion type you own.** `#[flags]` on `Permissions` enum generates
  `PermissionsFlags`: a concrete struct in your crate that you can `impl`.
- **Derive forwarding.** Any `#[derive(...)]` placed on the enum is
  automatically forwarded to both the enum and the generated flags struct.
- **Auto-sized backing integer.** The macro picks the smallest integer type
  that fits your variants; override it with `#[flags(u64)]` when needed.
- **`no_std` out of the box.** No allocator required.

## Installation

```toml
[dependencies]
flaglet = "0.1"
```

## Quickstart

```rust
use flaglet::flags;

#[flags]
pub enum Permissions {
    Read    = 1 << 0,
    Write   = 1 << 1,
    Execute = 1 << 2,
}

let mut perms = Permissions::flags(); // entry point on the enum itself
perms |= Permissions::Read;
perms |= Permissions::Write;

assert!(perms.contains(Permissions::Read));
assert!(!perms.contains(Permissions::Execute));
```

## Usage

### The `#[flags]` attribute

Apply `#[flags]` to any enum whose variants are unit variants with explicit
power-of-2 discriminants. The macro generates a companion struct named
`{EnumName}Flags` in the same scope.

```rust
use flaglet::flags;
use core::mem::size_of;

#[flags]
pub enum Permission {
    Read    = 0b001,
    Write   = 0b010,
    Execute = 0b100,
}
// Generates: pub struct PermissionFlags(u8);
assert!(PermissionFlags::empty().is_empty());
assert_eq!(size_of::<Permission>(), size_of::<u8>());
assert_eq!(size_of::<PermissionFlags>(), size_of::<u8>());
```

The backing integer type is chosen automatically from the largest
discriminant value (`u8`, `u16`, `u32`, or `u64`). You can override it
explicitly:

```rust
use flaglet::flags;
use core::mem::size_of;

#[flags(u32)]
pub enum Permission {
    Read    = 0b001,
    Write   = 0b010,
    Execute = 0b100,
}
// Generates: pub struct PermissionFlags(u32);
assert!(PermissionFlags::empty().is_empty());
assert_eq!(size_of::<Permission>(), size_of::<u32>());
assert_eq!(size_of::<PermissionFlags>(), size_of::<u32>());
```

### Constructing a flags value

```rust
use flaglet::flags;

#[flags]
enum Permissions {
    Read    = 1 << 0,
    Write   = 1 << 1,
    Execute = 1 << 2,
}

let _f = Permissions::flags();                           // empty
let _f = Permissions::all();                             // all variants set
let _f = Permissions::Read | Permissions::Write;         // combine variants
let _f = PermissionsFlags::from_flag(Permissions::Read); // single variant

// Const context — use union() instead of |
const RW: PermissionsFlags = PermissionsFlags::empty()
    .union(Permissions::Read)
    .union(Permissions::Write);

// From a raw integer (e.g. deserialization, FFI) — unknown bits are masked out
let _f = PermissionsFlags::from_bits(0b011_u8);
```

### Setting and testing flags

`set`, `unset`, `contains`, and `contains_any` accept both a single variant
and a combined flags value:

```rust
let mut f = PermissionsFlags::empty();
f.set(Permissions::Read);                        // set one flag
f.set(Permissions::Read | Permissions::Write);   // set multiple flags
f.unset(Permissions::Write);                     // clear one or more flags
let _ = f.is_empty();                            // true if no flags are set
let _ = f.bits();                                // raw integer value

// contains: true if ALL specified bits are set
let _ = f.contains(Permissions::Read);
let _ = f.contains(Permissions::Read | Permissions::Write);

// contains_any: true if AT LEAST ONE specified bit is set
let _ = f.contains_any(Permissions::Read | Permissions::Write);

// is_disjoint: true if NO specified bits are set (opposite of contains_any)
let _ = f.is_disjoint(Permissions::Execute);
```

### Bitwise operators

All standard bitwise operators are implemented between the enum and the
flags struct, in both mutating and non-mutating forms:

```rust
// Enum OP Enum → PermissionsFlags (chains freely)
let mut f = Permissions::Read | Permissions::Write | Permissions::Execute;

// Mutating
f |= Permissions::Read;    // set
f &= Permissions::Write;   // mask
f ^= Permissions::Execute; // toggle

// Non-mutating (returns a new PermissionsFlags)
let g = f | Permissions::Read;
let h = f & Permissions::Write;
let i = f ^ Permissions::Execute;
let j = !f; // invert all bits

// Flags OP Flags
let _k = f | g;
let _l = f & g;
```

### Equality

Cross-type `PartialEq` is implemented in both directions:

```rust
let mut f = PermissionsFlags::empty();
f.set(Permissions::Read);
assert!(f == Permissions::Read);   // PermissionsFlags == Permissions
assert!(Permissions::Read == f);   // Permissions == PermissionsFlags
```

Equality holds only when the flags value has exactly one bit set matching
the variant.

### Derive forwarding

Any `#[derive(...)]` on the enum is forwarded to **both** the enum and the
generated flags struct. This includes third-party derives like
`serde::Serialize` or `rkyv::Archive` — no stubs, no wrappers:

```rust
#[flags]
#[derive(Debug, Serialize, Deserialize, Archive)]
pub enum Permissions {
    Read    = 1 << 0,
    Write   = 1 << 1,
    Execute = 1 << 2,
}
// Both Permissions and PermissionsFlags implement Debug, Serialize,
// Deserialize, and Archive.
```

### Extending the generated type

Because `PermissionsFlags` is a real struct generated in your crate's
scope, you can add your own methods to it directly:

```rust
impl PermissionsFlags {
    pub fn read_write() -> Self {
        Permissions::flags() | Permissions::Read | Permissions::Write
    }

    pub fn is_read_only(&self) -> bool {
        self.contains(Permissions::Read) && !self.contains(Permissions::Write)
    }
}
```

### Visibility

The visibility of the enum is forwarded to both the enum and the generated
flags struct:

```rust
use flaglet::flags;

#[flags] pub enum Foo { A = 1 }        // pub enum Foo + pub struct FooFlags
#[flags] pub(crate) enum Bar { A = 1 } // pub(crate) for both
#[flags] enum Baz { A = 1 }            // private for both
```

### `no_std`

`flaglet` is fully `no_std` compatible. All generated code uses `core::ops`
exclusively. No feature flag required.

## Compile-time validation

The macro rejects invalid inputs with precise error messages pointing at
the offending token:

- Variants with fields (tuple or struct variants) are rejected.
- Variants without an explicit discriminant are rejected.
- Discriminants that are zero or not a power of 2 are rejected.
- Discriminants that exceed `u64::MAX` are rejected.
- Discriminant expressions must be integer literals or `1 << N` form.

## License

Licensed under either of [GPL-3.0-or-later](https://choosealicense.com/licenses/gpl-3.0/)
or [BSD-2-Clause](https://choosealicense.com/licenses/bsd-2-clause/) at your option.

<!-- cargo-rdme end -->
