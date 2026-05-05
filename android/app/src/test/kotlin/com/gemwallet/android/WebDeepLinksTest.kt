package com.gemwallet.android

import com.gemwallet.android.features.asset_select.presents.navigation.AssetsSearchRoute
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class WebDeepLinksTest {

    @Test
    fun webDeepLinkRoute_acceptsJoinLinks() {
        assertEquals(
            ReferralRoute(code = "gemcoder"),
            "https://gemwallet.com/join/gemcoder".toWebDeepLinkRoute(),
        )
        assertEquals(
            ReferralRoute(code = "gemcoder"),
            "https://gemwallet.com/join?code=gemcoder".toWebDeepLinkRoute(),
        )
        assertEquals(ReferralRoute(), "https://gemwallet.com/join".toWebDeepLinkRoute())
    }

    @Test
    fun webDeepLinkRoute_acceptsTokenLinks() {
        assertEquals(AssetsSearchRoute, "https://gemwallet.com/tokens".toWebDeepLinkRoute())
        assertEquals(
            AssetRoute(mockAssetId(Chain.Bitcoin)),
            "https://gemwallet.com/tokens/bitcoin".toWebDeepLinkRoute(),
        )
        assertEquals(
            AssetRoute(mockAssetId(Chain.Solana, tokenId = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN")),
            "https://gemwallet.com/tokens/solana/JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".toWebDeepLinkRoute(),
        )
    }

    @Test
    fun webDeepLinkRoute_rejectsUnsupportedLinks() {
        assertNull("https://gemwallet.com/swap/bitcoin".toWebDeepLinkRoute())
        assertNull("https://gemwallet.com/en/tokens/bitcoin".toWebDeepLinkRoute())
        assertNull("https://example.com/tokens/bitcoin".toWebDeepLinkRoute())
        assertNull("gem://tokens/bitcoin".toWebDeepLinkRoute())
        assertNull("https://gemwallet.com/tokens/notachain".toWebDeepLinkRoute())
        assertNull("https://gemwallet.com/tokens/bitcoin/too/many".toWebDeepLinkRoute())
    }
}
