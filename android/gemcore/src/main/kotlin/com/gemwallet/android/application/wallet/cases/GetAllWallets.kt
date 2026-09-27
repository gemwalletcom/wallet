package com.gemwallet.android.application.wallet.cases

import kotlinx.coroutines.flow.StateFlow
import uniffi.gemstone.GemWalletSection

interface GetAllWallets {
    fun getAllWallets(): StateFlow<List<GemWalletSection>>
}
