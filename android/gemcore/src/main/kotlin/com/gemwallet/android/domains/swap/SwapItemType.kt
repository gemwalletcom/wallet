package com.gemwallet.android.domains.swap

import kotlinx.serialization.Serializable
import uniffi.gemstone.GemSwapSide

@Serializable
enum class SwapItemType {
    Pay,
    Receive,
}

fun SwapItemType.toGem(): GemSwapSide = when (this) {
    SwapItemType.Pay -> GemSwapSide.PAY
    SwapItemType.Receive -> GemSwapSide.RECEIVE
}
