package com.gemwallet.android.application.wallet.coordinators

import com.wallet.core.primitives.WalletId

interface ToggleWalletPin {
    suspend fun toggleWalletPin(walletId: WalletId)
}
