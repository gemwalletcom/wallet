package com.gemwallet.android.application.wallet_connect.cases

import uniffi.gemstone.GemErrorText
import uniffi.gemstone.WalletConnectLink

interface PairWalletConnect {
    fun linkOpened(link: WalletConnectLink)
    fun pair(uri: String, onSuccess: () -> Unit = {}, onError: (GemErrorText) -> Unit = {})
}
