use jni_simple::jtype;

#[test]
pub fn test_float() {
    let x = jtype::from(28.123f32);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("float=2.8123e1"), "{}", format_str);
}

#[test]
pub fn test_byte() {
    let x = jtype::from(0x75i8);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("byte=0x75"));
}

#[test]
pub fn test_short() {
    let x = jtype::from(0x756i16);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("short=0x756"));
}

#[test]
pub fn test_int() {
    let x = jtype::from(0x756555i32);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("int=0x756555"));
}

#[test]
pub fn test_long() {
    let x = jtype::from(-0x7565554581458458i64);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("long=0x8a9aaaba7eba7ba8")); //2s complement
}

#[test]
pub fn test_double() {
    let x = jtype::from(756555.333221f64);
    let format_str = format!("{:?}", x);
    println!("{}", format_str);
    assert!(format_str.contains("double=7.56555333221e5"));
}
