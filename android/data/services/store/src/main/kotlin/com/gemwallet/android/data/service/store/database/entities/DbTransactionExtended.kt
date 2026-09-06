package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Embedded
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.VerificationStatus

data class DbTransactionExtended(
    @Embedded val transaction: DbTransaction,
    @Embedded(prefix = "asset_") val asset: DbAssetProjection,
    @Embedded(prefix = "fee_asset_") val feeAsset: DbAssetProjection,
    @ColumnInfo("price_value") val priceValue: Double?,
    @ColumnInfo("price_day_changed") val priceDayChanged: Double?,
    @ColumnInfo("fee_price_value") val feePriceValue: Double?,
    @ColumnInfo("fee_price_day_changed") val feePriceDayChanged: Double?,
    @ColumnInfo("from_price_value") val fromPriceValue: Double?,
    @ColumnInfo("from_price_day_changed") val fromPriceDayChanged: Double?,
    @ColumnInfo("to_price_value") val toPriceValue: Double?,
    @ColumnInfo("to_price_day_changed") val toPriceDayChanged: Double?,
    @Embedded(prefix = "from_asset_") val fromAsset: DbAssetProjection?,
    @Embedded(prefix = "to_asset_") val toAsset: DbAssetProjection?,
    @Embedded(prefix = "from_address_") val fromAddress: DbAddressProjection?,
    @Embedded(prefix = "to_address_") val toAddress: DbAddressProjection?,
)

data class DbAddressProjection(
    val chain: Chain,
    val name: String,
    val type: AddressType,
    val status: VerificationStatus,
)

fun DbTransactionExtended.toDTO(): TransactionExtended? {
    val swapFrom = fromAsset?.toDTO()
    val swapTo = toAsset?.toDTO()
    return TransactionExtended(
        transaction = transaction.toDTO(),
        asset = asset.toDTO() ?: return null,
        feeAsset = feeAsset.toDTO() ?: return null,
        price = priceValue?.let { Price(it, priceDayChanged ?: 0.0, 0L) },
        feePrice = feePriceValue?.let { Price(it, feePriceDayChanged ?: 0.0, 0L) },
        assets = listOfNotNull(swapFrom, swapTo),
        prices = listOfNotNull(
            swapFrom?.let { assetPrice(it.id, fromPriceValue, fromPriceDayChanged) },
            swapTo?.let { assetPrice(it.id, toPriceValue, toPriceDayChanged) },
        ),
        fromAddress = fromAddress?.toAddressName(transaction.owner),
        toAddress = toAddress?.toAddressName(transaction.recipient),
        confirmationEtaSeconds = transaction.confirmationEtaSeconds?.toUInt(),
    )
}

private fun assetPrice(assetId: AssetId, value: Double?, dayChanged: Double?): AssetPrice? =
    value?.let { AssetPrice(assetId, it, dayChanged ?: 0.0, 0L) }

private fun DbAddressProjection.toAddressName(address: String): AddressName = AddressName(
    chain = chain,
    address = address,
    name = name,
    type = type,
    status = status,
)

fun List<DbTransactionExtended>.toDTO() = mapNotNull { it.toDTO() }
