package com.gemwallet.android.ui.integration

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockSwapProviderData
import com.gemwallet.android.testkit.mockSwapQuote
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.Currency
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemSwapDetails
import uniffi.gemstone.GemSwapErrorDisplay
import uniffi.gemstone.GemSwapQuotesState
import uniffi.gemstone.swapQuoteDetails

@RunWith(AndroidJUnit4::class)
class SwapDetailsBottomSheetTest {
    @get:Rule
    val composeRule = createComposeRule()

    private val context = ApplicationProvider.getApplicationContext<Context>()
    private val details = swapQuoteDetails(
        mockSwapQuote(providerData = mockSwapProviderData(protocolName = "Uniswap")),
        mockAsset().toGem(),
        mockAsset().toGem(),
        null,
        null,
        Currency.USD.toGem(),
    )
    private var quotesState by mutableStateOf<GemSwapQuotesState>(GemSwapQuotesState.Quotes)
    private var shownDetails by mutableStateOf<GemSwapDetails?>(details)

    @Test
    fun aFailedRefreshShowsItsErrorInTheOpenSheet() {
        composeRule.setContent {
            WalletTheme {
                SwapDetailsBottomSheet(isVisible = true, details = shownDetails, quotesState = quotesState, onDismiss = {})
            }
        }
        composeRule.onNodeWithText("Uniswap").assertIsDisplayed()

        quotesState = GemSwapQuotesState.Failed(GemSwapErrorDisplay.NoQuote)
        shownDetails = null

        composeRule.onNodeWithText(context.getString(R.string.errors_swap_no_quote_available)).assertIsDisplayed()

        quotesState = GemSwapQuotesState.Quotes
        shownDetails = details

        composeRule.onNodeWithText("Uniswap").assertIsDisplayed()
    }
}
