package com.gemwallet.android.ui.models

import org.junit.Assert.assertEquals
import org.junit.Test

class ButtonStateTest {

    @Test
    fun loadingTakesPrecedenceOverEnabledAndDisabled() {
        assertEquals(ButtonState.Loading, buttonState(enabled = true, loading = true))
        assertEquals(ButtonState.Loading, buttonState(enabled = false, loading = true))
    }

    @Test
    fun enabledOnlyWhenEnabledAndNotLoading() {
        assertEquals(ButtonState.Enabled, buttonState(enabled = true, loading = false))
        assertEquals(ButtonState.Disabled, buttonState(enabled = false, loading = false))
    }

    @Test
    fun defaultsToEnabled() {
        assertEquals(ButtonState.Enabled, buttonState())
    }

    @Test
    fun onlyEnabledStateIsInteractive() {
        for (enabled in listOf(true, false)) {
            for (loading in listOf(true, false)) {
                val interactive = buttonState(enabled = enabled, loading = loading) == ButtonState.Enabled
                assertEquals(enabled && !loading, interactive)
            }
        }
    }
}
