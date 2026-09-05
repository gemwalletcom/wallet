package com.gemwallet.android.features.recipient.viewmodel.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.NFTAsset
import uniffi.gemstone.GemRecipientType

sealed interface RecipientType {
    val assetInfo: AssetInfo

    data class Asset(override val assetInfo: AssetInfo) : RecipientType

    data class Nft(
        override val assetInfo: AssetInfo,
        val nftAsset: NFTAsset,
    ) : RecipientType
}

sealed interface RecipientState {
    data object Loading : RecipientState
    data class Ready(val type: RecipientType) : RecipientState
}

fun RecipientType.toGem(): GemRecipientType = when (this) {
    is RecipientType.Asset -> GemRecipientType.Asset(assetInfo.asset.toGem())
    is RecipientType.Nft -> GemRecipientType.Nft(nftAsset.toGem())
}
