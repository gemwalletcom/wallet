package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.WalletId

interface SyncFiatTransactions {
    suspend operator fun invoke(walletId: WalletId? = null)
}
