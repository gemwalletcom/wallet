package com.gemwallet.android.features.wallet.presents.integration

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.features.wallet.presents.WalletDetailScene
import com.gemwallet.android.testkit.mockGemWalletDetails
import com.gemwallet.android.testkit.mockGemWalletRow
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class WalletDetailSceneTest {
    @get:Rule
    val composeRule = createComposeRule()

    private var wallet by mutableStateOf(mockGemWalletDetails(row = mockGemWalletRow(name = "Main")))

    @Test
    fun aSavedNameComingBackDoesNotOverwriteTheTypedName() {
        composeRule.setContent {
            WalletTheme {
                WalletDetailScene(wallet = wallet, onAction = {})
            }
        }
        composeRule.onNode(hasSetTextAction()).performTextInput(" wallet")

        wallet = mockGemWalletDetails(row = mockGemWalletRow(name = "Main w"))

        composeRule.onNode(hasSetTextAction()).assert(hasText("Main wallet"))
    }
}
