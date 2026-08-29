package com.gemwallet.android.application.bridge.cases

interface PairWalletConnect {
    fun pair(uri: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {})
}
