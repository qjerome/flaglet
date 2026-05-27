#![no_std]
use flaglet::flags;

#[flags]
enum Permissions {
    Read = 0b001,
    Write = 0b010,
    Execute = 0b100,
}

#[flags(u64)]
enum Explicit {
    A = 0b01,
    B = 0b10,
}

#[test]
fn explicit_type_override() {
    let f = ExplicitFlags::from_flag(Explicit::A);
    assert!(f.contains(Explicit::A));
    assert!(!f.contains(Explicit::B));
    assert_eq!(
        core::mem::size_of::<Explicit>(),
        core::mem::size_of::<u64>()
    )
}

#[test]
fn explicit_discriminants_are_preserved() {
    assert_eq!(Permissions::Read as usize, 0b001);
    assert_eq!(Permissions::Write as usize, 0b010);
    assert_eq!(Permissions::Execute as usize, 0b100);
}

#[test]
fn variants_are_disjoint() {
    assert_eq!(Permissions::Read as usize & Permissions::Write as usize, 0);
    assert_eq!(
        Permissions::Read as usize & Permissions::Execute as usize,
        0
    );
    assert_eq!(
        Permissions::Write as usize & Permissions::Execute as usize,
        0
    );
}

#[test]
fn mask_covers_all_variants() {
    assert_eq!(Permissions::all().bits(), 0b111);
}

#[test]
fn empty_has_no_flags() {
    let f = PermissionsFlags::empty();
    assert!(f.is_empty());
    assert!(!f.contains(Permissions::Read));
}

#[test]
fn from_flag() {
    let f = PermissionsFlags::from_flag(Permissions::Read);
    assert!(f.contains(Permissions::Read));
    assert!(!f.contains(Permissions::Write));
    assert!(!f.contains(Permissions::Execute));
}

#[test]
fn bitor_enum_chains() {
    let f = Permissions::Read | Permissions::Write | Permissions::Execute;
    assert!(f.contains(Permissions::Read));
    assert!(f.contains(Permissions::Write));
    assert!(f.contains(Permissions::Execute));
}

#[test]
fn from_bits_masks_unknown_bits() {
    let f = PermissionsFlags::from_bits(0xFF);
    assert!(f.contains(Permissions::Read));
    assert!(f.contains(Permissions::Write));
    assert!(f.contains(Permissions::Execute));
    assert_eq!(Permissions::all().bits(), 0b111);
    assert_eq!(f.bits(), 0b111);
}

#[test]
fn bits_returns_raw_value() {
    let f = Permissions::Read | Permissions::Write | Permissions::Execute;
    assert_eq!(f.bits(), 0b111);
}

#[test]
fn set_adds_flag() {
    let mut f = PermissionsFlags::empty();
    f.set(Permissions::Read);
    assert!(f.contains(Permissions::Read));
    assert!(!f.contains(Permissions::Write));
}

#[test]
fn unset_removes_flag() {
    let mut f = Permissions::Read | Permissions::Write;
    f.unset(Permissions::Read);
    assert!(!f.contains(Permissions::Read));
    assert!(f.contains(Permissions::Write));
}

#[test]
fn unset_noop_when_not_set() {
    let mut f = PermissionsFlags::from_flag(Permissions::Read);
    f.unset(Permissions::Write);
    assert!(f.contains(Permissions::Read));
    assert!(!f.contains(Permissions::Write));
}

#[test]
fn partial_eq() {
    let f = PermissionsFlags::from_flag(Permissions::Read);
    assert!(f == Permissions::Read);
    assert!(Permissions::Read == f);

    let g = Permissions::Read | Permissions::Write;
    assert!(g != Permissions::Read);
    assert!(g != Permissions::Write);
}

#[test]
fn bitor_assign_variant() {
    let mut f = PermissionsFlags::empty();
    f |= Permissions::Read;
    f |= Permissions::Write;
    assert!(f.contains(Permissions::Read));
    assert!(f.contains(Permissions::Write));
    assert!(!f.contains(Permissions::Execute));
}

#[test]
fn bitand_assign_variant() {
    let mut f = Permissions::Read | Permissions::Write;
    f &= Permissions::Read;
    assert!(f.contains(Permissions::Read));
    assert!(!f.contains(Permissions::Write));
}

#[test]
fn bitor_assign_flags() {
    let mut a = PermissionsFlags::from_flag(Permissions::Read);
    let b = PermissionsFlags::from_flag(Permissions::Write);
    a |= b;
    assert!(a.contains(Permissions::Read));
    assert!(a.contains(Permissions::Write));
}

#[test]
fn bitand_assign_flags() {
    let mut a = Permissions::Read | Permissions::Write;
    let b = Permissions::Write | Permissions::Execute;
    a &= b;
    assert!(!a.contains(Permissions::Read));
    assert!(a.contains(Permissions::Write));
    assert!(!a.contains(Permissions::Execute));
}

#[test]
fn bitxor_assign_variant_toggles() {
    let mut f = PermissionsFlags::from_flag(Permissions::Read);
    f ^= Permissions::Read;
    assert!(!f.contains(Permissions::Read));
    f ^= Permissions::Read;
    assert!(f.contains(Permissions::Read));
}

#[test]
fn bitxor_assign_flags_toggles() {
    let mut a = Permissions::Read | Permissions::Write;
    let b = PermissionsFlags::from_flag(Permissions::Write);
    a ^= b;
    assert!(a.contains(Permissions::Read));
    assert!(!a.contains(Permissions::Write));
}

#[test]
fn not_inverts_flags() {
    let f = PermissionsFlags::from_flag(Permissions::Read);
    let g = !f;
    assert!(!g.contains(Permissions::Read));
    assert!(g.contains(Permissions::Write));
    assert!(g.contains(Permissions::Execute));
}

#[test]
fn bitor_returns_new_value() {
    let f = PermissionsFlags::from_flag(Permissions::Read);
    let g = f | Permissions::Write;
    assert!(!f.contains(Permissions::Write));
    assert!(g.contains(Permissions::Read));
    assert!(g.contains(Permissions::Write));
}

#[test]
fn bitand_returns_new_value() {
    let f = Permissions::Read | Permissions::Write;
    let g = f & Permissions::Read;
    assert!(g.contains(Permissions::Read));
    assert!(!g.contains(Permissions::Write));
}

#[test]
fn bitxor_returns_new_value() {
    let f = Permissions::Read | Permissions::Write;
    let g = f ^ Permissions::Write;
    assert!(g.contains(Permissions::Read));
    assert!(!g.contains(Permissions::Write));
}

#[test]
fn contains() {
    let rw = Permissions::Read | Permissions::Write;
    let mut f = PermissionsFlags::from_flag(Permissions::Read);
    assert!(!f.contains(rw));
    f.set(Permissions::Write);
    assert!(f.contains(rw));
    assert!(f.contains(Permissions::Read | Permissions::Write));
    f.set(Permissions::Execute);
    assert!(f.contains(rw));
}

#[test]
fn contains_any() {
    let rw = Permissions::Read | Permissions::Write;
    let mut f = PermissionsFlags::empty();
    assert!(!f.contains_any(rw));
    f.set(Permissions::Read);
    assert!(f.contains_any(rw));
}

#[test]
fn is_disjoint_with_enum_variant() {
    let f = PermissionsFlags::from_flag(Permissions::Read);
    assert!(f.is_disjoint(Permissions::Write));
    assert!(f.is_disjoint(Permissions::Execute));
    assert!(!f.is_disjoint(Permissions::Read));
}

#[test]
fn is_disjoint_with_flags() {
    let f = PermissionsFlags::from_flag(Permissions::Read);
    let rw = Permissions::Read | Permissions::Write;
    assert!(!f.is_disjoint(rw));

    let we = Permissions::Write | Permissions::Execute;
    assert!(f.is_disjoint(we));
}

#[test]
fn empty_is_disjoint_with_everything() {
    let empty = PermissionsFlags::empty();
    assert!(empty.is_disjoint(Permissions::Read));
    assert!(empty.is_disjoint(Permissions::Write));
    assert!(empty.is_disjoint(Permissions::Execute));
    assert!(empty.is_disjoint(Permissions::all()));
}

#[test]
fn all_is_disjoint_with_nothing_nonempty() {
    let all = Permissions::all();
    assert!(!all.is_disjoint(Permissions::Read));
    assert!(!all.is_disjoint(Permissions::Write));
    assert!(!all.is_disjoint(Permissions::Execute));
}

#[test]
fn is_disjoint_complements_contains_any() {
    let f = Permissions::Read | Permissions::Write;
    let g = Permissions::Write | Permissions::Execute;
    assert_eq!(f.is_disjoint(g), !f.contains_any(g));

    let h = PermissionsFlags::from_flag(Permissions::Execute);
    assert_eq!(f.is_disjoint(h), !f.contains_any(h));
}

#[test]
fn clear_empties_all_flags() {
    let mut f = Permissions::Read | Permissions::Write | Permissions::Execute;
    f.clear();
    assert!(f.is_empty());
}

#[test]
fn clear_on_empty_is_noop() {
    let mut f = PermissionsFlags::empty();
    f.clear();
    assert!(f.is_empty());
}

#[test]
fn clear_then_set() {
    let mut f = Permissions::Read | Permissions::Write;
    f.clear();
    f.set(Permissions::Execute);
    assert!(!f.contains(Permissions::Read));
    assert!(!f.contains(Permissions::Write));
    assert!(f.contains(Permissions::Execute));
}

#[test]
fn union_is_const() {
    const RW: PermissionsFlags = PermissionsFlags::empty()
        .union(Permissions::Read)
        .union(Permissions::Write)
        .union(Permissions::Execute);

    assert!(RW.contains(Permissions::Read));
    assert!(RW.contains(Permissions::Write));
    assert!(RW.contains(Permissions::Execute));
}
