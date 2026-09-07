package com.gemwallet.android.testkit

import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionExtended
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
    confirmationEtaSeconds: UInt? = null,
    recordId: Long = 1,
) = TransactionExtended(
    recordId = recordId,
    transaction = transaction,
    asset = asset,
    feeAsset = feeAsset,
    price = price,
    feePrice = feePrice,
    assets = assets,
    prices = emptyList(),
    confirmationEtaSeconds = confirmationEtaSeconds,
)
