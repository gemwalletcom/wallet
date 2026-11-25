fn main() {
    #[cfg(feature = "kotlin-jni")]
    uniffi_bindgen_kotlin_jni::generate_scaffolding();
}
