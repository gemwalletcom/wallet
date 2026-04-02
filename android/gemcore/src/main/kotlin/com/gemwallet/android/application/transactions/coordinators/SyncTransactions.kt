package com.gemwallet.android.application.transactions.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

interface SyncTransactions {
    suspend fun syncTransactions(wallet: Wallet)
    suspend fun syncTransactions(wallet: Wallet, assetId: AssetId)
}
