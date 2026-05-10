package com.gemwallet.android.application.wallet_import.coordinators

import com.wallet.core.primitives.WalletId

interface SyncWalletConfiguration {
    suspend fun sync(walletId: WalletId)
}
