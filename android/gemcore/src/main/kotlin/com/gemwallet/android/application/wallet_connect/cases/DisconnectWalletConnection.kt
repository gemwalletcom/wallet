package com.gemwallet.android.application.wallet_connect.cases

interface DisconnectWalletConnection {
    suspend fun disconnect(connectionId: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {})
}
