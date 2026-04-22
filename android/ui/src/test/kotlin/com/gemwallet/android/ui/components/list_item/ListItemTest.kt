package com.gemwallet.android.ui.components.list_item

import org.junit.Assert.assertEquals
import org.junit.Test
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingMiddle

class ListItemTest {
    @Test
    fun contentSpacing_matchesSharedInnerPadding() {
        assertEquals(paddingMiddle, ListItemDefaults.contentSpacing)
    }

    @Test
    fun iconMinHeight_preservesIconAndPaddingContract() {
        assertEquals(listItemIconSize + paddingMiddle * 2, ListItemDefaults.iconMinHeight)
    }
}
