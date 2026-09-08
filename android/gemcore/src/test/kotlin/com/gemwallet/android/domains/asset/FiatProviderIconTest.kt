package com.gemwallet.android.domains.asset

import com.gemwallet.android.testkit.mockFiatProvider
import com.wallet.core.primitives.FiatProviderName
import org.junit.Assert.assertEquals
import org.junit.Test

class FiatProviderIconTest {

    @Test
    fun flashnetProvider_resolvesProviderName() {
        val provider = mockFiatProvider(
            id = FiatProviderName.Flashnet.string,
            name = "Cash App",
        )

        assertEquals(FiatProviderName.Flashnet, provider.providerName())
    }

    @Test
    fun unknownProvider_hasNoProviderName() {
        assertEquals(null, mockFiatProvider(id = "unknown", name = "Unknown").providerName())
    }
}
