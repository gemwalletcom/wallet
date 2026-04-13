package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.WalletId

interface GetFiatTransactions {
    suspend operator fun invoke(walletId: WalletId): List<FiatTransactionData>
}
