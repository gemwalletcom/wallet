package com.gemwallet.android.ui.integration

import android.content.Context
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemHeaderActions

@RunWith(AndroidJUnit4::class)
class AmountListHeadTest {

    @get:Rule
    val composeRule = createComposeRule()

    private val context: Context
        get() = ApplicationProvider.getApplicationContext()

    @Test
    fun watchWalletBanner_clickingBanner_showsWatchWalletInfoSheet() {
        setWatchWalletContent()

        composeRule.onNodeWithTag("watchWalletBanner").performClick()

        assertWatchWalletInfoSheetShown()
    }

    @Test
    fun watchWalletBanner_clickingInfoIcon_showsWatchWalletInfoSheet() {
        setWatchWalletContent()

        composeRule.onNodeWithTag("watchWalletInfo", useUnmergedTree = true).performClick()

        assertWatchWalletInfoSheetShown()
    }

    private fun setWatchWalletContent() {
        composeRule.setContent {
            WalletTheme {
                AssetHeadActions(actions = GemHeaderActions.WatchOnly, onTap = {})
            }
        }
    }

    private fun assertWatchWalletInfoSheetShown() {
        composeRule
            .onNodeWithText(context.getString(R.string.info_watch_wallet_title))
            .assertIsDisplayed()
    }
}
