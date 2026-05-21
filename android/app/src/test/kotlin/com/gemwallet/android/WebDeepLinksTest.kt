package com.gemwallet.android

import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.Deeplink

class WebDeepLinksTest {

    @Test
    fun toRoute_mapsSupportedDeeplinks() {
        val tokenId = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"
        assertEquals(AssetRoute(mockAssetId(Chain.Bitcoin)), Deeplink.Asset(assetId = "bitcoin").toRoute())
        assertEquals(AssetRoute(mockAssetId(Chain.Solana, tokenId)), Deeplink.Asset(assetId = "solana_$tokenId").toRoute())
        assertEquals(ReferralRoute(code = "gemcoder"), Deeplink.Rewards(code = "gemcoder").toRoute())
        assertEquals(ReferralRoute(), Deeplink.Rewards(code = null).toRoute())
    }

    @Test
    fun toRoute_rejectsUnsupportedLinks() {
        assertNull(Deeplink.Perpetuals.toRoute())
    }
}
