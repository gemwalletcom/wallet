package com.gemwallet.android.application.wallet.cases

import com.wallet.core.primitives.WalletId

interface SetWalletName {
    suspend fun setWalletName(walletId: WalletId, name: String)
}
