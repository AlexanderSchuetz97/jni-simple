use jni_simple::{jlong, jtype};

#[test]
pub fn test_float() {
    unsafe {
        let x = jtype::from(28.123f32);
        assert_eq!(28.123f32, x.float());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("float=2.8123e1"), "{}", format_str);
    }
}

#[test]
pub fn test_byte() {
    unsafe {
        let x = jtype::from(0x75i8);
        assert_eq!(0x75i8, x.byte());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("byte=0x75"));
    }
}

#[test]
pub fn test_short() {
    unsafe {
        let x = jtype::from(0x756i16);
        assert_eq!(0x756i16, x.short());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("short=0x756"));
    }
}

#[test]
pub fn test_char() {
    unsafe {
        let x = jtype::from(0x756u16);
        assert_eq!(0x756u16, x.char());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("short=0x756")); //We dont log the char value.
    }
}

#[test]
pub fn test_bool() {
    unsafe {
        let x = jtype::from(true);
        assert!(x.boolean());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);

        //This somewhat depends on the C-Compiler layout of the struct, which is different on big endian.
        #[cfg(target_endian = "little")]
        assert!(format_str.contains("byte=0x1"));

        let x = jtype::from(false);
        assert!(!x.boolean());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);

        //Struct is all 0's in any case. Endianess is irrelevant.
        assert!(format_str.contains("byte=0x0"));
    }
}

#[test]
pub fn test_int() {
    unsafe {
        let x = jtype::from(0x756555i32);
        assert_eq!(0x756555i32, x.int());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("int=0x756555"));
    }
}

#[test]
pub fn test_long() {
    unsafe {
        let x = jtype::from(-0x7565554581458458i64);
        assert_eq!(-0x7565554581458458i64, x.long());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("long=0x8a9aaaba7eba7ba8")); //2s complement
    }
}

#[test]
pub fn test_double() {
    unsafe {
        let x = jtype::from(756555.333221f64);
        assert_eq!(756555.333221f64, x.double());
        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("double=7.56555333221e5"));
    }
}

#[test]
pub fn test_pointers() {
    unsafe {
        let dangling = core::ptr::dangling_mut();

        let x = jtype::from(dangling);
        assert_eq!(dangling, x.object());
        assert_eq!(dangling, x.class());
        assert_eq!(dangling, x.throwable());
        assert_eq!(dangling as jlong, x.long());

        let null = core::ptr::null_mut();

        let x = jtype::from(null);
        assert_eq!(null, x.object());
        assert_eq!(null, x.class());
        assert_eq!(null, x.throwable());
        assert_eq!(0, x.long());

        let format_str = format!("{:?}", x.debug());
        println!("{}", format_str);
        assert!(format_str.contains("long=0x0"));
    }
}

#[test]
pub fn test_setter() {
    unsafe {
        let mut x = jtype::from(-1i64);
        //This should clear all other bits to 0
        x.set(-1i8);
        assert_eq!(0xFF, x.long());
        assert_eq!(-1, x.byte());
    }
}

#[test]
pub fn test_default_fmt() {
    let x = jtype::from(false);
    assert_eq!("jtype", format!("{x:?}").as_str());
}
