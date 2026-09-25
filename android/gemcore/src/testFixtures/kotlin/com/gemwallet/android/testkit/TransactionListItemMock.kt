package com.gemwallet.android.testkit

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionListItem

fun mockTransactionListItem(
    transaction: Transaction = mockTransaction(),
    asset: Asset = mockAsset(
        chain = transaction.assetId.chain,
        tokenId = transaction.assetId.tokenId,
    ),
    assets: List<Asset> = listOf(asset),
) = TransactionListItem(
    transaction = transaction,
    asset = asset,
    assets = assets,
)
