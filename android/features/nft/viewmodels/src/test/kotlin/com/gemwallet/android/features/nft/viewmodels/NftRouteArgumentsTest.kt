package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ui.models.navigation.RouteArgument
import org.junit.Assert.assertEquals
import org.junit.Test

class NftRouteArgumentsTest {

    @Test
    fun withoutArguments_listsCollections() {
        assertEquals(NftListMode.Collections, SavedStateHandle().nftListMode())
    }

    @Test
    fun withCollectionId_listsSingleCollection() {
        val savedStateHandle = SavedStateHandle(
            mapOf(RouteArgument.NftCollectionId.key to "ethereum_0x1")
        )

        assertEquals(NftListMode.Collection("ethereum_0x1"), savedStateHandle.nftListMode())
    }

    @Test
    fun withUnverifiedFlag_listsUnverifiedCollections() {
        val savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Unverified.key to true))

        assertEquals(NftListMode.Unverified, savedStateHandle.nftListMode())
    }
}
