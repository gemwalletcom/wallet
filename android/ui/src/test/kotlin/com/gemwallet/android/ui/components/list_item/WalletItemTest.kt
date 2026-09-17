package com.gemwallet.android.ui.components.list_item

import com.gemwallet.android.testkit.mockGemWalletRow
import com.gemwallet.android.ui.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class WalletItemTest {

    @Test
    fun `a row with the watch badge points at the badge drawable`() {
        val row = mockGemWalletRow(showsWatchBadge = true)

        assertEquals(
            "android.resource://com.gemwallet.android/drawable/${R.drawable.watch_badge}",
            row.supportIcon(),
        )
    }

    @Test
    fun `a row without the watch badge has no support icon`() {
        val row = mockGemWalletRow()

        assertNull(row.supportIcon())
    }
}
