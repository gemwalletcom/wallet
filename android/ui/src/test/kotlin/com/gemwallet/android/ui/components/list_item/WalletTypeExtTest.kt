package com.gemwallet.android.ui.components.list_item

import com.gemwallet.android.ui.R
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class WalletTypeExtTest {

    @Test
    fun `view wallets keep watch icon indicator`() {
        val supportIcon = WalletType.View.supportIcon()

        assertEquals(
            "android.resource://com.gemwallet.android/drawable/${R.drawable.watch_badge}",
            supportIcon,
        )
    }

    @Test
    fun `non view wallets show no watch icon indicator`() {
        val supportIcon = WalletType.Multicoin.supportIcon()

        assertNull(supportIcon)
    }
}
