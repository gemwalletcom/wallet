package com.gemwallet.android.application.wallet_connect.cases

import uniffi.gemstone.GemErrorText

interface PairWalletConnect {
    fun pair(uri: String, onSuccess: () -> Unit = {}, onError: (GemErrorText) -> Unit = {})
}
