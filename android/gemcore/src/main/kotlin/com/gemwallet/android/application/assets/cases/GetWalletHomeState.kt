package com.gemwallet.android.application.assets.cases

import kotlinx.coroutines.flow.StateFlow
import uniffi.gemstone.GemWalletHomeViewState

interface GetWalletHomeState {
    fun walletHomeState(): StateFlow<GemWalletHomeViewState?>
}
