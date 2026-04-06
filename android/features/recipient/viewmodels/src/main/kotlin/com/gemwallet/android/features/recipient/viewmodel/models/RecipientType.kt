package com.gemwallet.android.features.recipient.viewmodel.models

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.NFTAsset

sealed interface RecipientType {
    val assetInfo: AssetInfo

    data class Asset(override val assetInfo: AssetInfo) : RecipientType

    data class Nft(
        override val assetInfo: AssetInfo,
        val nftAsset: NFTAsset,
    ) : RecipientType
}
