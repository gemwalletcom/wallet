package com.gemwallet.android.ui.integration

import android.view.Window
import android.view.WindowManager
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.findActivity
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class DisableScreenShootingTest {

    @get:Rule
    val composeRule = createComposeRule()

    private var leaving by mutableStateOf(true)
    private var entering by mutableStateOf(false)
    private lateinit var window: Window

    private val isSecure: Boolean
        get() {
            composeRule.waitForIdle()
            return window.attributes.flags and WindowManager.LayoutParams.FLAG_SECURE != 0
        }

    @Test
    fun theWindowStaysSecureUntilTheLastSecretScreenLeaves() {
        composeRule.setContent {
            window = requireNotNull(LocalContext.current.findActivity()).window
            if (leaving) DisableScreenShooting()
            if (entering) DisableScreenShooting()
        }
        assertEquals(true, isSecure)

        entering = true
        assertEquals(true, isSecure)

        leaving = false
        assertEquals(true, isSecure)

        entering = false
        assertEquals(false, isSecure)
    }
}
