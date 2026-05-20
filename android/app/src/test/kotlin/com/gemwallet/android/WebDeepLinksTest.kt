package com.gemwallet.android

import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.Deeplink
import uniffi.gemstone.deeplinkDecodeUrl

class WebDeepLinksTest {

    @Before
    fun setUp() = mockkStatic("uniffi.gemstone.GemstoneKt")

    @After
    fun tearDown() = unmockkStatic("uniffi.gemstone.GemstoneKt")

    @Test
    fun webDeepLinkRoute_mapsSupportedDeeplinks() {
        val tokenId = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"
        every { deeplinkDecodeUrl(any()) } returns Deeplink.Asset(assetId = "bitcoin")
        assertEquals(AssetRoute(mockAssetId(Chain.Bitcoin)), "https://gemwallet.com/tokens/bitcoin".toWebDeepLinkRoute())

        every { deeplinkDecodeUrl(any()) } returns Deeplink.Asset(assetId = "solana_$tokenId")
        assertEquals(AssetRoute(mockAssetId(Chain.Solana, tokenId)), "gem://tokens/solana/$tokenId".toWebDeepLinkRoute())

        every { deeplinkDecodeUrl(any()) } returns Deeplink.Rewards(code = "gemcoder")
        assertEquals(ReferralRoute(code = "gemcoder"), "https://gemwallet.com/join/gemcoder".toWebDeepLinkRoute())

        every { deeplinkDecodeUrl(any()) } returns Deeplink.Rewards(code = null)
        assertEquals(ReferralRoute(), "https://gemwallet.com/join".toWebDeepLinkRoute())
    }

    @Test
    fun webDeepLinkRoute_rejectsUnsupportedLinks() {
        every { deeplinkDecodeUrl(any()) } returns Deeplink.Perpetuals
        assertNull("https://gemwallet.com/perpetuals".toWebDeepLinkRoute())

        every { deeplinkDecodeUrl(any()) } returns null
        assertNull("https://gemwallet.com/tokens".toWebDeepLinkRoute())
    }
}
