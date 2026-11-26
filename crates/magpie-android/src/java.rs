use std::ptr::null_mut;

use jni::{
    JNIEnv,
    objects::JString,
    sys::{JNI_FALSE, JNI_TRUE, jboolean, jbyteArray, jstring},
};

const ILLEGAL_STATE: &str = "java/lang/IllegalStateException";

pub fn text(env: &mut JNIEnv, value: &JString) -> Option<String> {
    env.get_string(value).ok().map(String::from)
}

pub fn string(env: &mut JNIEnv, value: &str) -> jstring {
    env.new_string(value)
        .map_or(null_mut(), |string| string.into_raw())
}

pub fn bytes(env: &mut JNIEnv, value: &[u8]) -> jbyteArray {
    env.byte_array_from_slice(value)
        .map_or(null_mut(), |array| array.into_raw())
}

pub fn truth(value: jboolean) -> bool {
    value == JNI_TRUE
}

pub fn boolean(value: bool) -> jboolean {
    match value {
        true => JNI_TRUE,
        false => JNI_FALSE,
    }
}

pub fn throw(env: &mut JNIEnv, message: &str) -> jstring {
    env.throw_new(ILLEGAL_STATE, message).ok();
    null_mut()
}
