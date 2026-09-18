package com.gemwallet.android.features.buy.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemFiatSession
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockFiatQuote
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.FiatQuoteUrl
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemServiceException
import java.math.BigInteger
import uniffi.gemstone.GemFiatSuggestedAmount

@OptIn(ExperimentalCoroutinesApi::class)
class FiatViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset()
    private val assetDataFlow = MutableStateFlow<AssetData?>(mockAssetData(price = mockAssetPriceInfo(price = 100.0)))

    private val getBuyAssetInfo = object : GetBuyAssetInfo {
        override fun invoke(assetId: AssetId): Flow<AssetData?> = assetDataFlow
    }
    private val assetPriceUsdFlow = MutableStateFlow<Double?>(100.0)
    private val getAssetPriceUsd = object : GetAssetPriceUsd {
        override fun invoke(assetId: AssetId): Flow<Double?> = assetPriceUsdFlow
    }
    private val fiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)
    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }
    private val service = mockk<GemFiatQuoteServiceInterface> {
        every { getCurrency() } returns Currency.USD.toGem()
        every { suggestedAmounts(any()) } returns listOf(GemFiatSuggestedAmount(100u, "$100"), GemFiatSuggestedAmount(250u, "$250"))
        every { newSession(any(), any()) } answers { mockGemFiatSession(firstArg(), secondArg()) }
        every { randomAmount() } returns 500u
        every { quoteDebounceMilliseconds() } returns 250uL
        every { quoteRefreshIntervalMilliseconds() } returns 300_000uL
        coEvery { quotes(any(), any(), any()) } returns listOf(mockFiatQuote())
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        unmockkStatic(Log::class)
        Dispatchers.resetMain()
    }

    @Test
    fun `buy quote is not refetched when only price changes`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assetDataFlow.value = mockAssetData(price = mockAssetPriceInfo(price = 125.0))
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            coVerify(exactly = 1) {
                service.quotes(FiatQuoteType.Buy.toGem(), asset.id.toIdentifier(), 50.0)
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `buy quote loads when asset data becomes available after init`() = runTest(testDispatcher) {
        assetDataFlow.value = null

        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            coVerify(exactly = 0) { service.quotes(any(), any(), any()) }

            assetDataFlow.value = mockAssetData(price = mockAssetPriceInfo(price = 100.0))
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            coVerify(exactly = 1) {
                service.quotes(FiatQuoteType.Buy.toGem(), asset.id.toIdentifier(), 50.0)
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `initial route amount overrides buy default`() = runTest(testDispatcher) {
        val viewModel = createViewModel(initialAmount = 10)

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertEquals("10", viewModel.amount.value)
            coVerify(exactly = 1) {
                service.quotes(FiatQuoteType.Buy.toGem(), asset.id.toIdentifier(), 10.0)
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `type change requests target operation amount`() = runTest(testDispatcher) {
        assetDataFlow.value = mockAssetData(balance = AssetBalance.create(asset, available = OneBitcoin), price = mockAssetPriceInfo(price = 100.0), metadata = mockAssetMetaData(isSellEnabled = true))
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            viewModel.setType(FiatQuoteType.Sell)
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertEquals("100", viewModel.amount.value)
            coVerify(exactly = 0) {
                service.quotes(FiatQuoteType.Sell.toGem(), asset.id.toIdentifier(), 50.0)
            }
            coVerify(exactly = 1) {
                service.quotes(FiatQuoteType.Sell.toGem(), asset.id.toIdentifier(), 100.0)
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `amount change clears current quote immediately`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertTrue(viewModel.providers.value.isNotEmpty())
            assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)

            viewModel.updateAmount("75")
            runCurrent()

            assertTrue(viewModel.providers.value.isEmpty())
            assertTrue(viewModel.uiState.value.isLoading)
            assertEquals(ButtonState.Loading, viewModel.uiState.value.buttonState)
            assertNull(viewModel.selectedProvider.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `quote request failure is distinct from an empty quote list and can retry`() = runTest(testDispatcher) {
        coEvery { service.quotes(any(), any(), any()) } throws GemServiceException.Api("offline")
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertEquals("string:${R.string.errors_unknown_try_again}", viewModel.uiState.value.errorText)
            assertTrue(viewModel.uiState.value.retries)
            assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)

            coEvery { service.quotes(any(), any(), any()) } returns listOf(mockFiatQuote())
            viewModel.retry()
            runCurrent()

            assertNull(viewModel.uiState.value.errorText)
            assertFalse(viewModel.uiState.value.retries)
            assertTrue(viewModel.providers.value.isNotEmpty())
            coVerify(exactly = 2) {
                service.quotes(FiatQuoteType.Buy.toGem(), asset.id.toIdentifier(), 50.0)
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `empty quote list remains quote not available`() = runTest(testDispatcher) {
        coEvery { service.quotes(any(), any(), any()) } returns emptyList()
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertEquals("string:${R.string.buy_no_results}", viewModel.uiState.value.errorText)
            assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `fiat type picker requires sell enabled metadata`() = runTest(testDispatcher) {
        assetDataFlow.value = mockAssetData(balance = AssetBalance.create(asset, available = OneBitcoin), price = mockAssetPriceInfo(price = 100.0), metadata = mockAssetMetaData(isSellEnabled = false))
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertFalse(viewModel.showFiatTypePicker.value)

            assetDataFlow.value = mockAssetData(balance = AssetBalance.create(asset, available = BigInteger("0")), price = mockAssetPriceInfo(price = 100.0), metadata = mockAssetMetaData(isSellEnabled = true))
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertTrue(viewModel.showFiatTypePicker.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `quote url failure is reported and releases the button`() = runTest(testDispatcher) {
        coEvery { service.quoteUrl(any(), any()) } throws GemServiceException.Api("offline")
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            val result = viewModel.quoteUrl()
            runCurrent()

            assertTrue(result.isFailure)
            assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `quote url success returns the provider redirect`() = runTest(testDispatcher) {
        coEvery { service.quoteUrl(any(), any()) } returns FiatQuoteUrl(redirectUrl = "https://provider.test/checkout", providerTransactionId = null)
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertEquals("https://provider.test/checkout", viewModel.quoteUrl().getOrThrow())
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `asset info balance includes symbol`() = runTest(testDispatcher) {
        assetDataFlow.value = mockAssetData(balance = AssetBalance.create(asset, available = OneBitcoin), price = mockAssetPriceInfo(price = 100.0))
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertEquals("1 BTC", viewModel.assetInfoUIModel.value?.balance)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `unsupported sell route falls back to buy with the requested amount`() = runTest(testDispatcher) {
        assetDataFlow.value = null
        val viewModel = createViewModel(initialAmount = 25, initialType = FiatQuoteType.Sell)

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertEquals(FiatQuoteType.Sell, viewModel.type.value)
            assertEquals("25", viewModel.amount.value)

            assetDataFlow.value = mockAssetData(balance = AssetBalance.create(asset, available = OneBitcoin), price = mockAssetPriceInfo(price = 100.0), metadata = mockAssetMetaData(isSellEnabled = false))
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertFalse(viewModel.showFiatTypePicker.value)
            assertEquals(FiatQuoteType.Buy, viewModel.type.value)
            assertEquals("25", viewModel.amount.value)
            coVerify(exactly = 1) {
                service.quotes(FiatQuoteType.Buy.toGem(), asset.id.toIdentifier(), 25.0)
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `random amount remains valid when current amount is at maximum`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            viewModel.updateAmount("1000")

            viewModel.updateAmount(FiatSuggestion.RandomAmount)
            runCurrent()

            assertEquals("500", viewModel.amount.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `provider fiat uses usd price source not session price`() = runTest(testDispatcher) {
        assetDataFlow.value = mockAssetData(price = mockAssetPriceInfo(price = 100.0))
        assetPriceUsdFlow.value = 200.0
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            val provider = viewModel.providers.value.first()
            assertEquals(fiatFormatter.string(200.0 * provider.cryptoAmount), provider.fiatFormatted)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    private fun createViewModel(initialAmount: Int? = null, initialType: FiatQuoteType? = null): FiatViewModel {
        val arguments = mutableMapOf<String, Any>(
            RouteArgument.AssetId.key to asset.id.toIdentifier(),
        )
        initialAmount?.let { arguments[RouteArgument.FiatAmount.key] = it }
        initialType?.let { arguments[RouteArgument.Type.key] = it }
        return FiatViewModel(
            getBuyAssetInfo = getBuyAssetInfo,
            getAssetPriceUsd = getAssetPriceUsd,
            service = service,
            context = context,
            ioDispatcher = testDispatcher,
            savedStateHandle = SavedStateHandle(arguments),
        )
    }

    private companion object {
        val OneBitcoin: BigInteger = BigInteger("100000000")
        const val DebounceSettleMs = 300L
    }
}
