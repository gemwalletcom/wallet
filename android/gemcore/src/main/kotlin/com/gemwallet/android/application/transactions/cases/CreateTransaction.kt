package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId

interface CreateTransaction {
    suspend fun trackTransactions(walletId: WalletId, transactions: List<Transaction>)

    fun trackPendingTransactions()

    suspend fun createNotificationTransaction(wallet: Wallet, assetId: AssetId, transaction: Transaction): Asset?
}
