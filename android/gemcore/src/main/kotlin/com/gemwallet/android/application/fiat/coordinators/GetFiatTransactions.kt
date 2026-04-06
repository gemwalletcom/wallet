package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.FiatTransactionData

interface GetFiatTransactions {
    suspend fun getFiatTransactions(walletId: String): List<FiatTransactionData>
}
