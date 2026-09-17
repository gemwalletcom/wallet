package com.gemwallet.android.features.recipient.viewmodel.models

import com.wallet.core.primitives.NFTAsset

sealed interface RecipientHeadUIModel {
    data class Nft(val nftAsset: NFTAsset) : RecipientHeadUIModel
    data object Asset : RecipientHeadUIModel
}
