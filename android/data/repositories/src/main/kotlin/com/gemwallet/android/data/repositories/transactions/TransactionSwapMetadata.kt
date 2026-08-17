package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTxSwapMetadata
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType

internal fun TransactionsDao.addSwapMetadata(transactions: List<Transaction>) {
    val swapMetadataRecords = transactions.mapNotNull { transaction ->
        if (transaction.type != TransactionType.Swap) {
            return@mapNotNull null
        }
        val metadata = transaction.metadata ?: return@mapNotNull null
        val swapMetadata = jsonEncoder.decodeFromString<TransactionSwapMetadata>(metadata)
        DbTxSwapMetadata(
            txId = transaction.id.identifier,
            fromAssetId = swapMetadata.fromAsset.toIdentifier(),
            toAssetId = swapMetadata.toAsset.toIdentifier(),
            fromAmount = swapMetadata.fromValue,
            toAmount = swapMetadata.toValue,
        )
    }
    addSwapMetadata(swapMetadataRecords)
}
