package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.Relation
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatTransactionAssetData
import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.FiatTransaction
import com.wallet.core.primitives.FiatTransactionStatus

@Entity(
    tableName = "fiat_transactions",
    primaryKeys = ["id", "walletId"],
    foreignKeys = [
        ForeignKey(DbWallet::class, ["id"], ["walletId"], onDelete = ForeignKey.CASCADE, onUpdate = ForeignKey.CASCADE),
        ForeignKey(DbAsset::class, ["id"], ["assetId"], onDelete = ForeignKey.CASCADE, onUpdate = ForeignKey.CASCADE),
    ],
    indices = [Index("walletId"), Index("assetId")],
)
data class DbFiatTransaction(
    val id: String,
    val walletId: String,
    val assetId: String,
    val transactionType: FiatQuoteType,
    val provider: FiatProviderName,
    val status: FiatTransactionStatus,
    val fiatAmount: Double,
    val fiatCurrency: String,
    val value: String,
    val createdAt: Long,
    val detailsUrl: String? = null,
)

data class DbFiatTransactionWithAsset(
    @Embedded val transaction: DbFiatTransaction,
    @Relation(parentColumn = "assetId", entityColumn = "id")
    val asset: DbAsset,
)

fun FiatTransactionData.toRecord(walletId: String) = DbFiatTransaction(
    id = transaction.id,
    walletId = walletId,
    assetId = transaction.assetId.toIdentifier(),
    transactionType = transaction.transactionType,
    provider = transaction.provider,
    status = transaction.status,
    fiatAmount = transaction.fiatAmount,
    fiatCurrency = transaction.fiatCurrency,
    value = transaction.value,
    createdAt = transaction.createdAt,
    detailsUrl = detailsUrl,
)

fun List<FiatTransactionData>.toRecord(walletId: String) = map { it.toRecord(walletId) }

fun DbFiatTransactionWithAsset.toDTO(): FiatTransactionAssetData? {
    val asset = asset.toDTO() ?: return null
    return FiatTransactionAssetData(
        transaction = FiatTransaction(
            id = transaction.id,
            assetId = asset.id,
            transactionType = transaction.transactionType,
            provider = transaction.provider,
            status = transaction.status,
            fiatAmount = transaction.fiatAmount,
            fiatCurrency = transaction.fiatCurrency,
            value = transaction.value,
            createdAt = transaction.createdAt,
        ),
        asset = asset,
        detailsUrl = transaction.detailsUrl,
    )
}

fun List<DbFiatTransactionWithAsset>.toDTO() = mapNotNull { it.toDTO() }
