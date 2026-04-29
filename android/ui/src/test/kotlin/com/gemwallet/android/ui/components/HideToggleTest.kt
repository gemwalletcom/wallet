package com.gemwallet.android.ui.components

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class HideToggleTest {
    @Test
    fun balanceIsHiddenOnlyWhenToggleIsOn() {
        assertFalse((null as HideToggle?).isHidden)
        assertFalse(HideToggle(hidden = false, onToggle = {}).isHidden)
        assertTrue(HideToggle(hidden = true, onToggle = {}).isHidden)
    }

    @Test
    fun hiddenBalanceIsReplacedWithFixedMask() {
        val visible = HideToggle(hidden = false, onToggle = {})
        val hidden = HideToggle(hidden = true, onToggle = {})

        assertEquals("\$1,234.56", (null as HideToggle?).mask("\$1,234.56"))
        assertEquals("\$1,234.56", visible.mask("\$1,234.56"))
        assertEquals("✱✱✱✱✱", hidden.mask("\$1,234.56"))
        assertEquals(hidden.mask("\$1.00"), hidden.mask("\$9,999,999.99"))
    }
}
