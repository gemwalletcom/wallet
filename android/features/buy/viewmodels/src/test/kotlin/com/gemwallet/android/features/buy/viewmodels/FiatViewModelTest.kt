package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.cases.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.cases.GetBuyQuotes
import com.gemwallet.android.domains.fiat.FiatConfig
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockFiatQuote
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.unmockkObject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import uniffi.gemstone.GemFiatServiceInterface
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class FiatViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset()
    private val wallet = mockWallet(id = "wallet-id")
    private val walletId = wallet.id
    private val assetDataFlow = MutableStateFlow<AssetData?>(assetData(price = 100.0))

    private val getBuyAssetInfo = object : GetBuyAssetInfo {
        override fun invoke(assetId: AssetId): Flow<AssetData?> = assetDataFlow
    }
    private val assetPriceUsdFlow = MutableStateFlow<Double?>(100.0)
    private val fiatService = mockk<GemFiatServiceInterface> {
        every { quoteDebounceMilliseconds() } returns 250uL
        every { quoteRefreshIntervalMilliseconds() } returns 300_000uL
    }

    private val getAssetPriceUsd = object : GetAssetPriceUsd {
        override fun invoke(assetId: AssetId): Flow<Double?> = assetPriceUsdFlow
    }
    private val fiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)
    private val getBuyQuotes = mockk<GetBuyQuotes>(relaxed = true) {
        coEvery {
            invoke(
                walletId = any(),
                asset = any(),
                type = any(),
                currency = any(),
                amount = any(),
            )
        } returns listOf(mockFiatQuote())
        coEvery {
            invoke(
                walletId = walletId,
                asset = asset,
                type = any(),
                currency = Currency.USD,
                amount = 50.0,
            )
        } returns listOf(mockFiatQuote())
        coEvery {
            invoke(
                walletId = walletId,
                asset = asset,
                type = FiatQuoteType.Sell,
                currency = Currency.USD,
                amount = 100.0,
            )
        } returns listOf(mockFiatQuote())
    }
    private val getBuyQuoteUrl = mockk<GetBuyQuoteUrl>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        mockkObject(FiatConfig)
        every { FiatConfig.defaultBuyAmount } returns 50
        every { FiatConfig.defaultSellAmount } returns 100
        every { FiatConfig.minimumAmount } returns 5
        every { FiatConfig.maximumAmount } returns 10000
        every { FiatConfig.randomMaxAmount } returns 1000
        every { FiatConfig.suggestedAmounts } returns listOf(100, 250)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        unmockkObject(FiatConfig)
    }

    @Test
    fun `buy quote is not refetched when only price changes`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            assetDataFlow.value = assetData(price = 125.0)
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            coVerify(exactly = 1) {
                getBuyQuotes(
                    walletId = walletId,
                    asset = asset,
                    type = FiatQuoteType.Buy,
                    currency = Currency.USD,
                    amount = 50.0,
                )
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

            assetDataFlow.value = assetData(price = 100.0)
            advanceTimeBy(DebounceSettleMs)
            runCurrent()

            coVerify(exactly = 1) {
                getBuyQuotes(
                    walletId = walletId,
                    asset = asset,
                    type = FiatQuoteType.Buy,
                    currency = Currency.USD,
                    amount = 50.0,
                )
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
                getBuyQuotes(
                    walletId = walletId,
                    asset = asset,
                    type = FiatQuoteType.Buy,
                    currency = Currency.USD,
                    amount = 10.0,
                )
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `fiat type picker requires sell enabled metadata`() = runTest(testDispatcher) {
        assetDataFlow.value = assetData(price = 100.0, isSellEnabled = false, available = OneBitcoin)
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertFalse(viewModel.showFiatTypePicker.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = true, available = "0")
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertTrue(viewModel.showFiatTypePicker.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `asset info balance includes symbol`() = runTest(testDispatcher) {
        assetDataFlow.value = assetData(price = 100.0, available = OneBitcoin)
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
    fun `sell type is only selected when sell is available`() = runTest(testDispatcher) {
        assetDataFlow.value = assetData(price = 100.0, isSellEnabled = false, available = "1")
        val viewModel = createViewModel()

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            viewModel.setType(FiatQuoteType.Sell)
            assertEquals(FiatQuoteType.Buy, viewModel.type.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = true, available = "0")
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            viewModel.setType(FiatQuoteType.Sell)
            assertEquals(FiatQuoteType.Sell, viewModel.type.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = false, available = OneBitcoin)
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertEquals(FiatQuoteType.Buy, viewModel.type.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `sell route arguments open sell with the requested amount`() = runTest(testDispatcher) {
        assetDataFlow.value = null
        val viewModel = createViewModel(initialAmount = 25, initialType = FiatQuoteType.Sell)

        try {
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertEquals(FiatQuoteType.Sell, viewModel.type.value)
            assertEquals("25", viewModel.amount.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = true, available = OneBitcoin)
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertEquals(FiatQuoteType.Sell, viewModel.type.value)
            assertEquals("25", viewModel.amount.value)

            viewModel.setType(FiatQuoteType.Buy)
            advanceTimeBy(DebounceSettleMs)
            runCurrent()
            assertEquals(FiatConfig.defaultBuyAmount.toString(), viewModel.amount.value)
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

            val randomAmount = viewModel.amount.value.toInt()
            assertTrue(randomAmount in FiatConfig.minimumAmount..FiatConfig.randomMaxAmount)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `provider fiat uses usd price source not session price`() = runTest(testDispatcher) {
        assetDataFlow.value = assetData(price = 100.0)
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
            getBuyQuotes = getBuyQuotes,
            getBuyQuoteUrl = getBuyQuoteUrl,
            getBuyAssetInfo = getBuyAssetInfo,
            getAssetPriceUsd = getAssetPriceUsd,
            fiatService = fiatService,
            savedStateHandle = SavedStateHandle(arguments),
        )
    }

    private fun assetData(
        price: Double,
        isSellEnabled: Boolean = false,
        available: String = "0",
    ) = mockAssetData(
        asset = asset,
        wallet = wallet,
        balance = AssetBalance.create(asset, available = available),
        metadata = mockAssetMetaData(isSellEnabled = isSellEnabled),
    ).copy(price = mockAssetPriceInfo(price = price))

    private companion object {
        const val OneBitcoin = "100000000"
        const val DebounceSettleMs = 300L
    }
}
