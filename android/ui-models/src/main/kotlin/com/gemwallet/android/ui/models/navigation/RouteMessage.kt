package com.gemwallet.android.ui.models.navigation

import uniffi.gemstone.GemErrorText

sealed interface RouteMessage {
    data class Toast(val text: String) : RouteMessage

    data class Error(val error: GemErrorText) : RouteMessage
}
