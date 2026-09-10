package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ui.models.navigation.RouteArgument
import uniffi.gemstone.GemNftList

internal fun SavedStateHandle.nftCollectionId(): String? = get<String>(RouteArgument.NftCollectionId.key)

internal fun SavedStateHandle.nftList(): GemNftList = when {
    nftCollectionId() != null -> GemNftList.COLLECTION
    get<Boolean>(RouteArgument.Unverified.key) == true -> GemNftList.UNVERIFIED
    else -> GemNftList.COLLECTIONS
}
