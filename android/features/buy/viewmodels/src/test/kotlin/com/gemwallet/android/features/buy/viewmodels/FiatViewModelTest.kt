package com.gemwallet.android.features.buy.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.gemwallet.android.data.services.store.queries.PriceUsdQuery
import com.gemwallet.android.domains.asset.aggregates.trailingValue
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBalance
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockAssetPrice
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockBalance
import com.gemwallet.android.testkit.mockFiatQuote
import com.gemwallet.android.testkit.mockFormattedNumber
import com.gemwallet.android.testkit.mockGemFiatSession
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
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
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemFiatSuggestedAmount
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.formattedCurrency
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class FiatViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(name = "Bitcoin", symbol = "BTC", decimals = 8)
    private val assetInfoFlow = MutableStateFlow<AssetInfo?>(mockAssetInfo(asset = asset, price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0))))

    private val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.Bitcoin)))
    private val sessionFlow = MutableStateFlow(mockSession(wallet = wallet))
    private val getSession = mockk<GetSession> {
        every { this@mockk.invoke() } returns sessionFlow
    }
    private val assetQuery = mockk<AssetQueryOptional> {
        every { this@mockk.invoke(wallet.id.id, asset.id) } returns assetInfoFlow
    }
    private val assetPriceUsdFlow = MutableStateFlow<Double?>(100.0)
    private val priceUsdQuery = mockk<PriceUsdQuery> {
        every { this@mockk.invoke(asset.id) } returns assetPriceUsdFlow
    }
    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }
    private val service = mockk<GemFiatQuoteServiceInterface> {
        every { suggestedAmounts() } returns listOf(GemFiatSuggestedAmount(100u, mockFormattedNumber(100.0)), GemFiatSuggestedAmount(250u, mockFormattedNumber(250.0)))
        every { newSession(any(), any()) } answers { mockGemFiatSession(firstArg(), secondArg()) }
        every { randomAmount() } returns 500u
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

            assetInfoFlow.value = mockAssetInfo(asset = asset, price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 125.0)))
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
        assetInfoFlow.value = null

        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            coVerify(exactly = 0) { service.quotes(any(), any(), any()) }

            assetInfoFlow.value = mockAssetInfo(asset = asset, price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)))
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
    fun `an asset on a chain the wallet has no account for is not offered`() = runTest(testDispatcher) {
        sessionFlow.value = mockSession(wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.Ethereum))))
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertNull(viewModel.assetInfoUIModel.value)
            coVerify(exactly = 0) { service.quotes(any(), any(), any()) }
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
        assetInfoFlow.value =
            mockAssetInfo(
                asset = asset,
                balance = mockAssetBalance(asset = asset, balance = mockBalance(available = OneBitcoin)),
                price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)),
                metadata = mockAssetMetaData(isSellEnabled = true),
            )
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

            assertEquals("offline", viewModel.uiState.value.quotesMessage)
            assertTrue(viewModel.uiState.value.retries)
            assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)

            coEvery { service.quotes(any(), any(), any()) } returns listOf(mockFiatQuote())
            viewModel.retry()
            runCurrent()

            assertNull(viewModel.uiState.value.quotesMessage)
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
    fun `an empty amount asks to enter an amount to buy`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            viewModel.updateAmount("")
            runCurrent()

            assertEquals("string:${R.string.input_enter_amount_to}", viewModel.uiState.value.quotesMessage)
            verify { context.getString(R.string.input_enter_amount_to, "string:${R.string.wallet_buy}") }
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

            assertEquals("string:${R.string.buy_no_results}", viewModel.uiState.value.quotesMessage)
            assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a sell quote above the balance keeps the providers and moves the error onto the amount`() = runTest(testDispatcher) {
        assetInfoFlow.value = mockAssetInfo(asset = asset, price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)), metadata = mockAssetMetaData(isSellEnabled = true))
        coEvery { service.quotes(any(), any(), any()) } returns listOf(mockFiatQuote(quoteType = FiatQuoteType.Sell, asset = asset))
        val viewModel = createViewModel(initialType = FiatQuoteType.Sell)

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertTrue(viewModel.providers.value.isNotEmpty())
            assertNull(viewModel.uiState.value.quotesMessage)
            assertEquals("string:${R.string.transfer_insufficient_balance}", viewModel.uiState.value.amountError)
            assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `fiat type picker requires sell enabled metadata`() = runTest(testDispatcher) {
        assetInfoFlow.value =
            mockAssetInfo(
                asset = asset,
                balance = mockAssetBalance(asset = asset, balance = mockBalance(available = OneBitcoin)),
                price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)),
                metadata = mockAssetMetaData(isSellEnabled = false),
            )
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertFalse(viewModel.showsTypePicker.value)

            assetInfoFlow.value =
                mockAssetInfo(
                    asset = asset,
                    balance = mockAssetBalance(asset = asset, balance = mockBalance(available = BigInteger("0"))),
                    price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)),
                    metadata = mockAssetMetaData(isSellEnabled = true),
                )
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertTrue(viewModel.showsTypePicker.value)
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
        assetInfoFlow.value = mockAssetInfo(asset = asset, balance = mockAssetBalance(asset = asset, balance = mockBalance(available = OneBitcoin)), price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)))
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assertEquals("1 BTC", (viewModel.assetInfoUIModel.value?.row?.trailingValue?.text as? GemLocalizedText.Number)?.number?.text())
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `unsupported sell route falls back to buy with the requested amount`() = runTest(testDispatcher) {
        assetInfoFlow.value = null
        val viewModel = createViewModel(initialAmount = 25, initialType = FiatQuoteType.Sell)

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertEquals(FiatQuoteType.Sell, viewModel.type.value)
            assertEquals("25", viewModel.amount.value)

            assetInfoFlow.value =
                mockAssetInfo(
                    asset = asset,
                    balance = mockAssetBalance(asset = asset, balance = mockBalance(available = OneBitcoin)),
                    price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)),
                    metadata = mockAssetMetaData(isSellEnabled = false),
                )
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertFalse(viewModel.showsTypePicker.value)
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

            viewModel.selectRandomAmount()
            runCurrent()

            assertEquals("500", viewModel.amount.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `provider fiat uses usd price source not session price`() = runTest(testDispatcher) {
        assetInfoFlow.value = mockAssetInfo(asset = asset, price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0)))
        assetPriceUsdFlow.value = 200.0
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            val provider = viewModel.providers.value.first()
            assertEquals(formattedCurrency(200.0 * provider.row.cryptoAmount.value, Currency.USD.string, GemCurrencyStyle.FIAT).text(), provider.fiatFormatted)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `the quote clock only runs while the screen is started`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            coVerify(exactly = 1) { service.quotes(any(), any(), any()) }

            advanceTimeBy(RefreshIntervalMs + DebounceSettleMs)
            runCurrent()
            coVerify(exactly = 1) { service.quotes(any(), any(), any()) }

            viewModel.setRefreshEnabled(true)
            advanceTimeBy(RefreshIntervalMs + DebounceSettleMs)
            runCurrent()
            coVerify(exactly = 2) { service.quotes(any(), any(), any()) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed quote stops the clock until the amount changes`() = runTest(testDispatcher) {
        coEvery { service.quotes(any(), any(), any()) } throws GemServiceException.Api("offline")
        val viewModel = createViewModel()

        try {
            viewModel.setRefreshEnabled(true)
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            coVerify(exactly = 1) { service.quotes(any(), any(), any()) }

            advanceTimeBy(RefreshIntervalMs * 2)
            runCurrent()
            coVerify(exactly = 1) { service.quotes(any(), any(), any()) }

            viewModel.updateAmount("75")
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            coVerify(exactly = 1) { service.quotes(any(), any(), 75.0) }
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
            getSession = getSession,
            assetQuery = assetQuery,
            priceUsdQuery = priceUsdQuery,
            service = service,
            context = context,
            ioDispatcher = testDispatcher,
            savedStateHandle = SavedStateHandle(arguments),
        )
    }

    private companion object {
        val OneBitcoin: BigInteger = BigInteger("100000000")
        const val DebounceSettleMs = 300L
        const val RefreshIntervalMs = 300_000L
    }
}
