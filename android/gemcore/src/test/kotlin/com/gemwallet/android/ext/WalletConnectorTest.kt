package com.gemwallet.android.ext

import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class WalletConnectorTest {
    @Test
    fun iconUrlUsesProxyForStoredRelativeIcon() {
        val metadata = ApplicationMetadata(
            name = "App",
            description = "",
            url = "https://app.example.com/swap/",
            icon = "../icon.svg?v=1&theme=dark",
            source = ApplicationMetadataSource.WalletConnect,
        )

        assertEquals(
            "https://assets.gemwallet.com/proxy/image?url=https%3A%2F%2Fapp.example.com%2Ficon.svg%3Fv%3D1%26theme%3Ddark&size=256",
            metadata.iconUrl,
        )
        assertNull(metadata.copy(icon = "http://app.example.com/icon.png").iconUrl)
    }
}
