package com.gemwallet.android.application.wallet_connect.cases

import uniffi.gemstone.GemErrorText

interface DisconnectWalletConnection {
    suspend fun disconnect(connectionId: String, onSuccess: () -> Unit = {}, onError: (GemErrorText) -> Unit = {})
}
