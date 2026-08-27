package com.gemwallet.android.cases.transactions

import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.WalletId

interface CreateTransaction {
    suspend fun createTransaction(walletId: WalletId, transaction: Transaction): Transaction
}
