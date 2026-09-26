package com.gemwallet.android.features.transfer.viewmodels.recipient.models

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemRecipientType

sealed interface RecipientUIState {
    data object Loading : RecipientUIState
    data class Ready(val asset: Asset, val type: GemRecipientType) : RecipientUIState {
        val head: RecipientHeadUIModel
            get() = when (type) {
                is GemRecipientType.Nft -> RecipientHeadUIModel.Nft(type.nftAsset.toPrimitives())
                is GemRecipientType.Asset -> RecipientHeadUIModel.Asset
            }
    }
}
