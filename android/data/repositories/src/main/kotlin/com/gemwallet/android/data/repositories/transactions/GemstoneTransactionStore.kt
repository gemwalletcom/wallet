package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemTransactionStore

class GemstoneTransactionStore(
    private val walletPreferencesFactory: WalletPreferencesFactory,
    private val saveTransactions: SaveTransactions,
) : GemTransactionStore {

    override suspend fun getSyncTimestamp(walletId: String, assetId: String?): ULong {
        val preferences = walletPreferencesFactory.create(walletId)
        return (assetId?.let { preferences.transactionsForAssetTimestamp(it) } ?: preferences.transactionsTimestamp).toULong()
    }

    override suspend fun setSyncTimestamp(walletId: String, assetId: String?, timestamp: ULong) {
        val preferences = walletPreferencesFactory.create(walletId)
        if (assetId == null) {
            preferences.transactionsTimestamp = timestamp.toLong()
        } else {
            preferences.setTransactionsForAssetTimestamp(assetId, timestamp.toLong())
        }
    }

    override suspend fun saveTransactions(walletId: String, transactions: List<String>) =
        saveTransactions.saveTransactions(WalletId(walletId), transactions.map { it.decodeJson<Transaction>() })
}
