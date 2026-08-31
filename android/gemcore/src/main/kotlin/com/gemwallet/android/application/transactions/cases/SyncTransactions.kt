package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.Wallet

interface SyncTransactions {
    suspend fun syncTransactions(wallet: Wallet): Boolean
}
