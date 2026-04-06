package com.gemwallet.android.ui.components.list_item

import androidx.compose.ui.unit.dp
import org.junit.Assert.assertEquals
import org.junit.Test

class ListItemTest {
    @Test
    fun defaultTrailingContentEndPadding_keepsDefaultInsetForCompactSpacing() {
        assertEquals(16.dp, defaultTrailingContentEndPadding(8.dp))
    }

    @Test
    fun defaultTrailingContentEndPadding_preservesLargerInsets() {
        assertEquals(20.dp, defaultTrailingContentEndPadding(20.dp))
    }
}
