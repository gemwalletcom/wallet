package com.gemwallet.android.data.service.store.database.entities

import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.WalletId

fun mockDbTransactionExtended(
    transaction: Transaction,
    walletId: WalletId = WalletId("wallet-1"),
    asset: DbAssetProjection = mockDbAssetProjection(),
    feeAsset: DbAssetProjection = mockDbAssetProjection(),
) = DbTransactionExtended(
    transaction = transaction.toRecord(walletId),
    asset = asset,
    feeAsset = feeAsset,
    priceValue = null,
    priceDayChanged = null,
    feePriceValue = null,
    feePriceDayChanged = null,
    fromAsset = null,
    toAsset = null,
    fromAddress = null,
    toAddress = null,
)

fun mockDbAssetProjection(
    id: String = "bitcoin",
    name: String = "Asset",
    symbol: String = "A",
    decimals: Int = 8,
    type: AssetType = AssetType.NATIVE,
) = DbAssetProjection(
    id = id,
    name = name,
    symbol = symbol,
    decimals = decimals,
    type = type,
)
