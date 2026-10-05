package com.gemwallet.android.features.transfer.presents.integration

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.features.transfer.presents.confirm.components.FeeDetails
import com.gemwallet.android.testkit.mockGemCustomFeeSession
import com.gemwallet.android.testkit.mockGemFeeRateRow
import com.gemwallet.android.testkit.mockGemFeeRateRows
import com.gemwallet.android.testkit.mockGemNetworkFeeScreen
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemFeeRateKind
import uniffi.gemstone.GemListRowTitle

@RunWith(AndroidJUnit4::class)
class FeeDetailsTest {
    @get:Rule
    val composeRule = createComposeRule()

    private val custom = mockGemCustomFeeSession(
        rows = mockGemFeeRateRows(
            rows = listOf(mockGemFeeRateRow(kind = GemFeeRateKind.Custom, title = GemListRowTitle.CUSTOM_FEE)),
            showsOptions = true,
        ),
    )
    private var screen by mutableStateOf(mockGemNetworkFeeScreen(rates = custom.rows, custom = custom))

    @Test
    fun typedCustomFeeSurvivesRefresh() {
        composeRule.setContent {
            WalletTheme {
                FeeDetails(
                    isVisible = true,
                    screen = screen,
                    feeListItem = null,
                    onSelectPriority = {},
                    onSelectCustom = {},
                    onSelectFeeAsset = {},
                    onCancel = {},
                )
            }
        }
        composeRule.onNodeWithText(ApplicationProvider.getApplicationContext<Context>().getString(R.string.fee_rate_custom)).performClick()
        composeRule.onNode(hasSetTextAction()).performTextInput("25")

        screen = mockGemNetworkFeeScreen(rates = custom.rows, custom = custom.copy(price = 2.0))

        composeRule.onNode(hasSetTextAction()).assertTextEquals("25")
    }
}
