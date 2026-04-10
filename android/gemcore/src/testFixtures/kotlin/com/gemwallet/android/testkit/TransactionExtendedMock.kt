package com.gemwallet.android.testkit

import com.gemwallet.android.model.Transaction
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Price

fun mockTransactionExtended(
    transaction: Transaction = mockTransaction(),
    asset: Asset = mockAsset(
        chain = transaction.assetId.chain,
        tokenId = transaction.assetId.tokenId,
    ),
    feeAsset: Asset = asset,
    price: Price? = null,
    feePrice: Price? = null,
    assets: List<Asset> = listOf(asset),
) = TransactionExtended(
    transaction = transaction,
    asset = asset,
    feeAsset = feeAsset,
    price = price,
    feePrice = feePrice,
    assets = assets,
)
