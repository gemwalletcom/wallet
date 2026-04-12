package com.gemwallet.android.ext

import org.junit.Assert.assertEquals
import org.junit.Test

class UpdateUrlTest {

    @Test
    fun `universal flavor uses versioned apk url`() {
        assertEquals(
            "https://apk.gemwallet.com/gem_wallet_universal_2.29.apk",
            updateUrl(
                flavor = "universal",
                version = "2.29",
                fallbackUrl = "https://apk.gemwallet.com/gem_wallet_latest.apk",
            )
        )
    }

    @Test
    fun `non universal flavors keep fallback url`() {
        assertEquals(
            "https://play.google.com/store/apps/details?id=com.gemwallet.android",
            updateUrl(
                flavor = "google",
                version = "2.29",
                fallbackUrl = "https://play.google.com/store/apps/details?id=com.gemwallet.android",
            )
        )
    }
}
