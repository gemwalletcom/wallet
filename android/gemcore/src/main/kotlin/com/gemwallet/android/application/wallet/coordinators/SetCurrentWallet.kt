package com.gemwallet.android.application.wallet.coordinators

import com.wallet.core.primitives.WalletId

interface SetCurrentWallet {
    suspend fun setCurrentWallet(walletId: WalletId)
}
