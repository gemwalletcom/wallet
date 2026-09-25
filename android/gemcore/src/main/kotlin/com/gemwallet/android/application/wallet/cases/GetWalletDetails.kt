package com.gemwallet.android.application.wallet.cases

import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemWalletDetails

interface GetWalletDetails {
    fun getWallet(walletId: WalletId): Flow<GemWalletDetails?>
}
