use flaglet::flags;

#[flags]
#[derive(Debug)]
#[allow(dead_code)]
enum Status {
    Active = 0b0001,
    Inactive = 0b0010,
    Pending = 1 << 2,
}

#[test]
fn derived_traits_forwarded_to_enum_and_flags() {
    let variant = Status::Active;
    let mut f = Status::flags();
    f.set(Status::Active);
    assert_eq!(format!("{:?}", variant), "Active");
    assert_eq!(format!("{:?}", f), "StatusFlags(1)");
}
