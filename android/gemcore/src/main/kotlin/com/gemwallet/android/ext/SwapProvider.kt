package com.gemwallet.android.ext

import com.wallet.core.primitives.SwapProvider

fun String.toSwapProvider(): SwapProvider? {
    return SwapProvider.entries.firstOrNull { it.string == this }
}
