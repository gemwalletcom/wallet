package com.gemwallet.android.testkit

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionExtended

fun mockTransactionExtended(
    transaction: Transaction = mockTransaction(),
    asset: Asset = mockAsset(
        id = transaction.assetId,
    ),
    feeAsset: Asset = asset,
    assets: List<Asset> = listOf(asset),
) = TransactionExtended(
    recordId = 1,
    transaction = transaction,
    asset = asset,
    feeAsset = feeAsset,
    price = null,
    feePrice = null,
    assets = assets,
    prices = emptyList(),
    confirmationEtaSeconds = null,
)
