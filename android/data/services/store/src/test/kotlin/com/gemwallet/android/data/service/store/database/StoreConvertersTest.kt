package com.gemwallet.android.data.service.store.database

import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.NFTAttribute
import org.junit.Assert.assertEquals
import org.junit.Test

class StoreConvertersTest {
    private val converters = StoreConverters()

    @Test
    fun assetLinks_roundTripAsJson() {
        val links = listOf(
            AssetLink(name = "website", url = "https://gemwallet.com"),
            AssetLink(name = "x", url = "https://x.com/gemwallet"),
        )

        assertEquals(links, converters.toAssetLinks(converters.fromAssetLinks(links)))
    }

    @Test
    fun nftAttributes_roundTripAsJson() {
        val attributes = listOf(
            NFTAttribute(name = "Background", value = "Blue"),
            NFTAttribute(name = "Strength", value = "Legendary", percentage = 1.2),
        )

        assertEquals(attributes, converters.toNftAttributes(converters.fromNftAttributes(attributes)))
    }
}
