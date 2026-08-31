package com.gemwallet.android.application.wallet.cases

import com.wallet.core.primitives.WalletId

interface SetCurrentWallet {
    suspend fun setCurrentWallet(walletId: WalletId)
}
