package com.gemwallet.android.cases.transactions

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

interface SyncTransactions {
    suspend fun syncTransactions(wallet: Wallet)
    suspend fun syncTransactions(wallet: Wallet, assetId: AssetId)
}