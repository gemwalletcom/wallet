package com.gemwallet.android.features.activities.presents.details

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.NFTAssetId

sealed interface TransactionDetailsAction {
    sealed interface Navigation : TransactionDetailsAction

    data object Close : Navigation
    data class OpenAsset(val assetId: AssetId) : Navigation
    data class OpenNft(val assetId: NFTAssetId) : Navigation
    data class OpenPerpetual(val assetId: AssetId) : Navigation
    data class OpenSwap(val fromAssetId: AssetId, val toAssetId: AssetId) : Navigation

    data object Share : TransactionDetailsAction
    data object ShowFeeDetails : TransactionDetailsAction
    data class OpenAddress(val chainAddress: ChainAddress) : TransactionDetailsAction
}
