package com.gemwallet.android.data.services.store.database.entities

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId

fun mockDbTransactionExtended(type: TransactionType = TransactionType.Transfer, priceValue: Double? = null, assets: List<Asset> = emptyList(), prices: List<DbPrice> = emptyList()): DbTransactionExtended {
    val asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    val id = TransactionId(asset.id.chain, "0xhash")
    return DbTransactionExtended(
        transaction = DbTransaction(
            id = id,
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
        asset = asset.toDbAssetProjection(),
        feeAsset = asset.toDbAssetProjection(),
        priceValue = priceValue,
        priceDayChanged = null,
        feePriceValue = null,
        feePriceDayChanged = null,
        fromAddress = null,
        toAddress = null,
        transactionKey = id.identifier,
        assets = assets.map { it.toDbAssetProjection() },
        prices = prices,
    )
}

private fun Asset.toDbAssetProjection() = DbAssetProjection(
    id = id.toIdentifier(),
    name = name,
    symbol = symbol,
    decimals = decimals,
    type = type,
)
