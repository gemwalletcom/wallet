package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemTransactionStore

class GemstoneTransactionStore(
    private val saveTransactions: SaveTransactions,
) : GemTransactionStore {

    override suspend fun saveTransactions(walletId: String, transactions: List<String>) =
        saveTransactions.saveTransactions(WalletId(walletId), transactions.map { it.decodeJson<Transaction>() })
}
