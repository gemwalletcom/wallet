package com.gemwallet.android.application.wallet.coordinators

import com.wallet.core.primitives.WalletId

interface DeleteWallet {
    suspend fun deleteWallet(
        walletId: WalletId,
        onBoard: () -> Unit,
        onComplete: () -> Unit,
    )
}
