package com.gemwallet.android.ui.theme

import org.junit.Assert.assertEquals
import org.junit.Test

class SceneContentPaddingTest {
    @Test
    fun compactWidthUsesMiddlePadding() {
        assertEquals(paddingMiddle, sceneContentPadding(isCompactWidth = true))
    }

    @Test
    fun regularWidthUsesDefaultPadding() {
        assertEquals(paddingDefault, sceneContentPadding(isCompactWidth = false))
    }
}
