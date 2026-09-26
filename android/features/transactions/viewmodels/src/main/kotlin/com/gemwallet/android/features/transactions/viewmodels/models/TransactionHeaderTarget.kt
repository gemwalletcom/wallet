package com.gemwallet.android.features.transactions.viewmodels.models

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAssetId
import uniffi.gemstone.GemTransactionHeaderAction

sealed interface TransactionHeaderTarget {
    data class Asset(val assetId: AssetId) : TransactionHeaderTarget
    data class Nft(val assetId: NFTAssetId) : TransactionHeaderTarget
    data class Swap(val fromAssetId: AssetId, val toAssetId: AssetId) : TransactionHeaderTarget
    data class Perpetual(val assetId: AssetId) : TransactionHeaderTarget
}

internal fun GemTransactionHeaderAction.target(): TransactionHeaderTarget = when (this) {
    is GemTransactionHeaderAction.Asset -> TransactionHeaderTarget.Asset(AssetId(assetId))
    is GemTransactionHeaderAction.Nft -> TransactionHeaderTarget.Nft(NFTAssetId(assetId))
    is GemTransactionHeaderAction.Swap -> TransactionHeaderTarget.Swap(fromAssetId = AssetId(fromAssetId), toAssetId = AssetId(toAssetId))
    is GemTransactionHeaderAction.Perpetual -> TransactionHeaderTarget.Perpetual(AssetId(assetId))
}
