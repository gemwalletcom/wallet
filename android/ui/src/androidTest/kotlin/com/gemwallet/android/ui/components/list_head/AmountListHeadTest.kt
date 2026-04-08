package com.gemwallet.android.ui.components.list_head

import android.content.Context
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.test.core.app.ApplicationProvider
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.WalletType
import org.junit.Rule
import org.junit.Test

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
                AssetHeadActions(
                    walletType = WalletType.View,
                    transferEnabled = false,
                    operationsEnabled = false,
                    onTransfer = null,
                    onReceive = null,
                    onBuy = null,
                    onSwap = null,
                )
            }
        }
    }

    private fun assertWatchWalletInfoSheetShown() {
        composeRule
            .onNodeWithText(context.getString(R.string.info_watch_wallet_title))
            .assertIsDisplayed()
    }
}
