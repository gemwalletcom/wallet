package com.gemwallet.android.ext

import com.wallet.core.primitives.FeePriority

fun String.toFeePriority(): FeePriority? {
    return FeePriority.entries.firstOrNull { it.string == this }
}
