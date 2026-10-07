package uniffi.gemstone

import com.sun.jna.Callback
import com.sun.jna.CallbackReference

fun initializeForeignFutureResultStructs() {
    CallbackReference.getFunctionPointer(ForeignFutureResultStructs)
}

private object ForeignFutureResultStructs : Callback {
    fun callback(
        u8: UniffiForeignFutureResultU8.UniffiByValue,
        i8: UniffiForeignFutureResultI8.UniffiByValue,
        u16: UniffiForeignFutureResultU16.UniffiByValue,
        i16: UniffiForeignFutureResultI16.UniffiByValue,
        u32: UniffiForeignFutureResultU32.UniffiByValue,
        i32: UniffiForeignFutureResultI32.UniffiByValue,
        u64: UniffiForeignFutureResultU64.UniffiByValue,
        i64: UniffiForeignFutureResultI64.UniffiByValue,
        f32: UniffiForeignFutureResultF32.UniffiByValue,
        f64: UniffiForeignFutureResultF64.UniffiByValue,
        rustBuffer: UniffiForeignFutureResultRustBuffer.UniffiByValue,
        void: UniffiForeignFutureResultVoid.UniffiByValue,
    ) = Unit
}
