package com.gemwallet.android.features.fiat_connect.presents.integration

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.features.fiat_connect.presents.FiatProvidersList
import com.gemwallet.android.testkit.mockGemProviderRow
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.FiatProviderName
import uniffi.gemstone.GemProviderKind

@RunWith(AndroidJUnit4::class)
class FiatProvidersListTest {
    @get:Rule
    val composeRule = createComposeRule()

    private val isShow = mutableStateOf(true)
    private var providers by mutableStateOf(listOf(mockGemProviderRow(kind = GemProviderKind.Fiat(FiatProviderName.MOON_PAY), name = "MoonPay")))

    @Test
    fun theOpenListKeepsItsProvidersWhileQuotesRefresh() {
        composeRule.setContent {
            WalletTheme {
                FiatProvidersList(isShow = isShow, providers = providers, onProviderSelect = {})
            }
        }
        composeRule.onNodeWithText("MoonPay").assertIsDisplayed()

        providers = emptyList()

        composeRule.onNodeWithText("MoonPay").assertIsDisplayed()
    }
}
