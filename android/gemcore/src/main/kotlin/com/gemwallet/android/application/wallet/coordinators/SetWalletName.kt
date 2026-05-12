package com.gemwallet.android.application.wallet.coordinators

import com.wallet.core.primitives.WalletId

interface SetWalletName {
    suspend fun setWalletName(walletId: WalletId, name: String)
}
