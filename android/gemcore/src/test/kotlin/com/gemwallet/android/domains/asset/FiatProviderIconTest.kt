package com.gemwallet.android.domains.asset

import com.gemwallet.android.testkit.mockFiatProvider
import com.wallet.core.primitives.FiatProviderName
import org.junit.Assert.assertEquals
import org.junit.Test

class FiatProviderIconTest {

    @Test
    fun flashnetProvider_usesFlashnetAsset() {
        val provider = mockFiatProvider(
            id = FiatProviderName.Flashnet.string,
            name = "Cash App",
        )

        assertEquals(
            "file:///android_asset/fiat/flashnet.svg",
            provider.getFiatProviderIcon(),
        )
    }

    @Test
    fun flashnetProviderName_usesFlashnetAsset() {
        assertEquals(
            "file:///android_asset/fiat/flashnet.svg",
            FiatProviderName.Flashnet.getFiatProviderIcon(),
        )
    }
}
