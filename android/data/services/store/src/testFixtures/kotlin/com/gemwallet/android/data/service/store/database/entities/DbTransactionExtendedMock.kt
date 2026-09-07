package com.gemwallet.android.data.service.store.database.entities

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId

fun mockDbAssetProjection(asset: Asset) = DbAssetProjection(
    id = asset.id.toIdentifier(),
    name = asset.name,
    symbol = asset.symbol,
    decimals = asset.decimals,
    type = asset.type,
)

fun mockDbTransactionExtended(
    asset: Asset = Asset(
        id = AssetId(chain = Chain.Ethereum),
        name = "Ethereum",
        symbol = "ETH",
        decimals = 18,
        type = AssetType.NATIVE,
    ),
    type: TransactionType = TransactionType.Transfer,
    priceValue: Double? = null,
    fromAsset: Asset? = null,
    toAsset: Asset? = null,
    fromPriceValue: Double? = null,
    fromPriceDayChanged: Double? = null,
    toPriceValue: Double? = null,
    toPriceDayChanged: Double? = null,
) = DbTransactionExtended(
    transaction = DbTransaction(
        id = TransactionId(asset.id.chain, "0xhash"),
        walletId = WalletId("wallet-1"),
        hash = "0xhash",
        assetId = asset.id,
        feeAssetId = AssetId(chain = asset.id.chain),
        owner = "owner",
        recipient = "recipient",
        state = TransactionState.Confirmed,
        type = type,
        blockNumber = "1",
        sequence = "1",
        fee = "1",
        value = "5",
        direction = TransactionDirection.Outgoing,
        createdAt = 0,
        updatedAt = 0,
    ),
    asset = mockDbAssetProjection(asset),
    feeAsset = mockDbAssetProjection(asset),
    priceValue = priceValue,
    priceDayChanged = null,
    feePriceValue = null,
    feePriceDayChanged = null,
    fromPriceValue = fromPriceValue,
    fromPriceDayChanged = fromPriceDayChanged,
    toPriceValue = toPriceValue,
    toPriceDayChanged = toPriceDayChanged,
    fromAsset = fromAsset?.let(::mockDbAssetProjection),
    toAsset = toAsset?.let(::mockDbAssetProjection),
    fromAddress = null,
    toAddress = null,
)
