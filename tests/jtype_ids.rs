use jni_simple::*;

#[test]
fn test() {
    assert_eq!('Z', jboolean::jtype_id());
    assert_eq!('B', jbyte::jtype_id());
    assert_eq!('S', jshort::jtype_id());
    assert_eq!('C', jchar::jtype_id());
    assert_eq!('I', jint::jtype_id());
    assert_eq!('J', jlong::jtype_id());
    assert_eq!('F', jfloat::jtype_id());
    assert_eq!('D', jdouble::jtype_id());
    assert_eq!('L', jobject::jtype_id());
}
