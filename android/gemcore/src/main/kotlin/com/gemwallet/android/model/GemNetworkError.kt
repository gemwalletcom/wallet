package com.gemwallet.android.model

sealed class GemNetworkError {
    data object Offline : GemNetworkError()
    data class Display(val message: String) : GemNetworkError()
    data class Generic(val message: String) : GemNetworkError()
}
