package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.FiatTransactionInfo

interface GetFiatTransactions {
    suspend fun getFiatTransactions(walletId: String): List<FiatTransactionInfo>
}
