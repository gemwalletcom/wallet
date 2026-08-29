package com.gemwallet.android.application.wallet_connect.cases

interface PairWalletConnect {
    fun pair(uri: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {})
}
