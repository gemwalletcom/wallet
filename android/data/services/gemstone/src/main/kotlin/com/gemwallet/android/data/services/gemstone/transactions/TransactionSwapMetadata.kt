package com.gemwallet.android.data.services.gemstone.transactions

import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransactionSwapMetadata
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Transaction
import uniffi.gemstone.transactionSwapPair

internal fun TransactionsDao.addSwapMetadata(transactions: List<Transaction>) {
    val swapMetadataRecords = transactions.mapNotNull { transaction ->
        transactionSwapPair(transaction.toGem())?.let { pair ->
            DbTransactionSwapMetadata(
                transactionId = transaction.id.identifier,
                fromAssetId = pair.fromAssetId,
                toAssetId = pair.toAssetId,
            )
        }
    }
    addSwapMetadata(swapMetadataRecords)
}
