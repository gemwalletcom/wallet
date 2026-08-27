package com.gemwallet.android.features.bridge.viewmodels.model

sealed class BridgeRequestError(message: String) : Exception(message) {
    object MaliciousSession : BridgeRequestError("Malicious session")
}
