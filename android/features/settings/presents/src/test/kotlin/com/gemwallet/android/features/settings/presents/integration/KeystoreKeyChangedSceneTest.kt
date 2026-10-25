package com.gemwallet.android.features.settings.presents.integration

import android.app.Application
import android.content.Intent
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.settings.presents.lock.KeystoreKeyChangedScene
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Shadows.shadowOf
import uniffi.gemstone.PublicUrl

@RunWith(AndroidJUnit4::class)
class KeystoreKeyChangedSceneTest {

    @get:Rule
    val composeRule = createComposeRule()

    private val application = ApplicationProvider.getApplicationContext<Application>()

    @Test
    fun supportOpensTheWebSupportPage() {
        var isReset = false
        composeRule.setContent { WalletTheme { KeystoreKeyChangedScene(onReset = { isReset = true }) } }

        composeRule.onNodeWithText(application.getString(R.string.settings_support)).performClick()

        val opened = shadowOf(application).nextStartedActivity
        assertEquals(Intent.ACTION_VIEW, opened.action)
        assertEquals(AppUrl.page(PublicUrl.SUPPORT), opened.dataString)
        assertFalse(isReset)
    }
}
