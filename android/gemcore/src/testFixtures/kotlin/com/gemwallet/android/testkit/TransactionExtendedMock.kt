package com.gemwallet.android.testkit

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionExtended

fun mockTransactionExtended(
    transaction: Transaction = mockTransaction(),
    asset: Asset = mockAsset(
        chain = transaction.assetId.chain,
        tokenId = transaction.assetId.tokenId,
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
