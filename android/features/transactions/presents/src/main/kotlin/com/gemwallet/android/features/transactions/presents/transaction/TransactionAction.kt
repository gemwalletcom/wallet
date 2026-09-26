package com.gemwallet.android.features.transactions.presents.transaction

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.NFTAssetId

sealed interface TransactionAction {
    sealed interface Navigation : TransactionAction

    data object Close : Navigation
    data class OpenAsset(val assetId: AssetId) : Navigation
    data class OpenNft(val assetId: NFTAssetId) : Navigation
    data class OpenPerpetual(val assetId: AssetId) : Navigation
    data class OpenSwap(val fromAssetId: AssetId, val toAssetId: AssetId) : Navigation
    data class OpenAddress(val chainAddress: ChainAddress) : Navigation

    data object Share : TransactionAction
    data object ShowFeeDetails : TransactionAction
}
