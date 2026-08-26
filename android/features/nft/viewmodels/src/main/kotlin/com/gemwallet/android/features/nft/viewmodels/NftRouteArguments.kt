package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ui.models.navigation.RouteArgument

internal fun SavedStateHandle.nftListMode(): NftListMode {
    val collectionId = get<String>(RouteArgument.NftCollectionId.key)
    return when {
        collectionId != null -> NftListMode.Collection(collectionId)
        get<Boolean>(RouteArgument.Unverified.key) == true -> NftListMode.Unverified
        else -> NftListMode.Collections
    }
}
