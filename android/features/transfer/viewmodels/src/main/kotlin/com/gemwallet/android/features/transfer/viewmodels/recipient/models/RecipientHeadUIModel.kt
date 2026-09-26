package com.gemwallet.android.features.transfer.viewmodels.recipient.models

import com.wallet.core.primitives.NFTAsset

sealed interface RecipientHeadUIModel {
    data class Nft(val nftAsset: NFTAsset) : RecipientHeadUIModel
    data object Asset : RecipientHeadUIModel
}
