package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Embedded
import androidx.room.Junction
import androidx.room.Relation
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.VerificationStatus

data class DbTransactionExtended(
    @Embedded val transaction: DbTransaction,
    @Embedded(prefix = "asset_") val asset: DbAssetProjection,
    @Embedded(prefix = "fee_asset_") val feeAsset: DbAssetProjection,
    @ColumnInfo("price_value") val priceValue: Double?,
    @ColumnInfo("price_day_changed") val priceDayChanged: Double?,
    @ColumnInfo("fee_price_value") val feePriceValue: Double?,
    @ColumnInfo("fee_price_day_changed") val feePriceDayChanged: Double?,
    @Embedded(prefix = "from_address_") val fromAddress: DbAddressProjection?,
    @Embedded(prefix = "to_address_") val toAddress: DbAddressProjection?,
    @ColumnInfo("tx_key") val transactionKey: String,
    @Relation(
        entity = DbAsset::class,
        parentColumn = "tx_key",
        entityColumn = "id",
        associateBy = Junction(DbTransactionAsset::class, parentColumn = "tx_id", entityColumn = "asset_id"),
    )
    val assets: List<DbAssetProjection>,
    @Relation(
        parentColumn = "tx_key",
        entityColumn = "asset_id",
        associateBy = Junction(DbTransactionAsset::class, parentColumn = "tx_id", entityColumn = "asset_id"),
    )
    val prices: List<DbPrice>,
)

data class DbAddressProjection(val chain: Chain, val name: String, val type: AddressType, val status: VerificationStatus)

fun DbTransactionExtended.toDTO(): TransactionExtended? {
    return TransactionExtended(
        recordId = transaction.recordId,
        transaction = transaction.toDTO(),
        asset = asset.toDTO() ?: return null,
        feeAsset = feeAsset.toDTO() ?: return null,
        price = priceValue?.let { Price(it, priceDayChanged ?: 0.0, 0L) },
        feePrice = feePriceValue?.let { Price(it, feePriceDayChanged ?: 0.0, 0L) },
        assets = assets.mapNotNull { it.toDTO() },
        prices = prices.mapNotNull { it.toAssetPrice() },
        fromAddress = fromAddress?.toAddressName(transaction.owner),
        toAddress = toAddress?.toAddressName(transaction.recipient),
        confirmationEtaSeconds = transaction.confirmationEtaSeconds?.toUInt(),
    )
}

private fun DbPrice.toAssetPrice(): AssetPrice? = value?.let { AssetPrice(assetId.toAssetId() ?: return null, it, dayChanged ?: 0.0, 0L) }

internal fun DbAddressProjection.toAddressName(address: String): AddressName = AddressName(
    chain = chain,
    address = address,
    name = name,
    type = type,
    status = status,
)
