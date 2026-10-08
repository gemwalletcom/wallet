package com.gemwallet.android.ui.components.image

import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.components.infoSheetIconSize
import com.gemwallet.android.ui.theme.listItemIconSize
import org.junit.Assert.assertEquals
import org.junit.Test

class IconWithBadgeTest {
    @Test
    fun badgeLayout_smallIconsUseTheCompactRatio() {
        val layout = badgeLayout(listItemIconSize)

        assertEquals(listItemIconSize / 2.6f, layout.contentSize)
        assertEquals(layout.contentSize + 4.dp, layout.badgeSize)
    }

    @Test
    fun badgeLayout_largeIconsUseTheLargeRatio() {
        val layout = badgeLayout(infoSheetIconSize)

        assertEquals(infoSheetIconSize / 3f, layout.contentSize)
        assertEquals(layout.contentSize + 4.dp, layout.badgeSize)
        assertEquals(5.dp, layout.offset)
    }
}
