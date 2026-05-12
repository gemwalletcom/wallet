package com.gemwallet.android.ui.components

import com.gemwallet.android.ui.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class InfoSheetEntityTest {

    @Test
    fun watchWalletInfo_usesStandaloneWatchIcon() {
        assertEquals(R.drawable.watch_badge, InfoSheetEntity.WatchWalletInfo.icon)
        assertNull(InfoSheetEntity.WatchWalletInfo.badgeIcon)
    }
}
