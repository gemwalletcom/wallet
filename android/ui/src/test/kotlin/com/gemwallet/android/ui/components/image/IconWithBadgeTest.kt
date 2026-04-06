package com.gemwallet.android.ui.components.image

import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.components.infoSheetIconSize
import com.gemwallet.android.ui.theme.headerIconSize
import com.gemwallet.android.ui.theme.listItemIconSize
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class IconWithBadgeTest {
    @Test
    fun badgeLayout_leavesVisibleRingAroundBadgeContent() {
        val layout = badgeLayout(listItemIconSize)

        assertTrue(layout.contentSize < layout.badgeSize)
        assertTrue(layout.ringWidth > 0.dp)
        assertEquals(layout.contentSize + layout.ringWidth * 2, layout.badgeSize)
    }

    @Test
    fun badgeLayout_offsetsByOuterBadgeSize() {
        val layout = badgeLayout(headerIconSize)

        assertEquals(layout.badgeSize / 5f, layout.offset)
    }

    @Test
    fun badgeLayout_keepsSmallIconSizingRule() {
        val layout = badgeLayout(listItemIconSize)

        assertEquals(listItemIconSize / 2.6f, layout.contentSize)
        assertEquals(listItemIconSize / 32f, layout.ringWidth)
    }

    @Test
    fun badgeLayout_capsRingWidthForInfoSheetIcons() {
        val layout = badgeLayout(infoSheetIconSize)

        assertEquals(infoSheetIconSize / 3f, layout.contentSize)
        assertEquals(2.dp, layout.ringWidth)
        assertTrue(layout.badgeSize < infoSheetIconSize)
        assertEquals(layout.contentSize + layout.ringWidth * 2, layout.badgeSize)
    }
}
