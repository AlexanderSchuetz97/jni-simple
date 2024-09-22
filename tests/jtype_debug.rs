use jni_simple::jtype;

#[test]
pub fn test_float() {
    let x = jtype::from(28.123f32);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("float=2.8123e1"), "{}", format_str);
}

#[test]
pub fn test_short() {
    let x = jtype::from(0x756i16);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("short=0x756"));
}