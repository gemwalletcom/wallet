package com.gemwallet.android.ext

import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.FeePriority as GemFeePriority

fun GemFeePriority.toPrimitives(): FeePriority = when (this) {
    GemFeePriority.NORMAL -> FeePriority.Normal
    GemFeePriority.FAST -> FeePriority.Fast
}

fun FeePriority.toGem(): GemFeePriority = when (this) {
    FeePriority.Normal -> GemFeePriority.NORMAL
    FeePriority.Fast -> GemFeePriority.FAST
}
