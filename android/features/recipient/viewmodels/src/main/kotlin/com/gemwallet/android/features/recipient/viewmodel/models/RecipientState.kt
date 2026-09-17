package com.gemwallet.android.features.recipient.viewmodel.models

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemRecipientType

sealed interface RecipientState {
    data object Loading : RecipientState
    data class Ready(val asset: Asset, val type: GemRecipientType) : RecipientState {
        val head: RecipientHeadUIModel
            get() = when (type) {
                is GemRecipientType.Nft -> RecipientHeadUIModel.Nft(type.nftAsset.toPrimitives())
                is GemRecipientType.Asset -> RecipientHeadUIModel.Asset
            }
    }
}
