package com.gemwallet.android.ext

import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class WalletConnectorTest {
    @Test
    fun iconUrlUsesWebsiteOrigin() {
        val metadata = ApplicationMetadata(
            name = "App",
            description = "",
            url = "https://app.example.com/swap/?theme=dark#section",
            icon = "../icon.svg?v=1&theme=dark",
            source = ApplicationMetadataSource.WalletConnect,
        )

        val expected = "https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Fapp.example.com&size=256"
        assertEquals(expected, metadata.iconUrl)
        assertEquals(expected, metadata.copy(icon = "").iconUrl)
        assertNull(metadata.copy(url = "http://app.example.com").iconUrl)
    }
}
