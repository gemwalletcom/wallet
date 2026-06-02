package com.gemwallet.android.application.wallet.coordinators

import com.wallet.core.primitives.WalletId

interface SetWalletAvatar {
    suspend fun setWalletAvatar(walletId: WalletId, imageUrl: String?)
}
