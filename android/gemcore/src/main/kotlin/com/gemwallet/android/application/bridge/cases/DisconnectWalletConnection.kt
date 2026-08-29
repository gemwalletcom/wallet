package com.gemwallet.android.application.bridge.cases

interface DisconnectWalletConnection {
    suspend fun disconnect(connectionId: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {})
}
