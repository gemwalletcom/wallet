package com.gemwallet.android.features.nft.viewmodels.models

import com.wallet.core.primitives.NFTAssetData
import uniffi.gemstone.GemCollectibleDetails

data class CollectibleUIModel(val assetData: NFTAssetData, val details: GemCollectibleDetails)
