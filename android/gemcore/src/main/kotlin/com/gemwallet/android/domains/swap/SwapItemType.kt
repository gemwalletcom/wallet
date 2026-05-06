package com.gemwallet.android.domains.swap

import kotlinx.serialization.Serializable

@Serializable
enum class SwapItemType {
    Pay,
    Receive,
}
