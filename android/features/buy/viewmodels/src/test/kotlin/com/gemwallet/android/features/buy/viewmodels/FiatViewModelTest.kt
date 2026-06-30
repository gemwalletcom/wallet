package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.domains.fiat.FiatConfig
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetData
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
import com.wallet.core.primitives.WalletId
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
import kotlinx.coroutines.test.runCurrent
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
    private val getBuyQuotes = mockk<GetBuyQuotes>(relaxed = true) {
        coEvery {
            invoke(
                walletId = any(),
                asset = any(),
                type = any(),
                fiatCurrency = any(),
                amount = any(),
            )
        } returns listOf(mockFiatQuote())
        coEvery {
            invoke(
                walletId = walletId,
                asset = asset,
                type = any(),
                fiatCurrency = Currency.USD.string,
                amount = 50.0,
            )
        } returns listOf(mockFiatQuote())
        coEvery {
            invoke(
                walletId = walletId,
                asset = asset,
                type = FiatQuoteType.Sell,
                fiatCurrency = Currency.USD.string,
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
            runCurrent()

            assetDataFlow.value = assetData(price = 125.0)
            runCurrent()

            coVerify(exactly = 1) {
                getBuyQuotes(
                    walletId = walletId,
                    asset = asset,
                    type = FiatQuoteType.Buy,
                    fiatCurrency = Currency.USD.string,
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
            runCurrent()

            assetDataFlow.value = assetData(price = 100.0)
            runCurrent()

            coVerify(exactly = 1) {
                getBuyQuotes(
                    walletId = walletId,
                    asset = asset,
                    type = FiatQuoteType.Buy,
                    fiatCurrency = Currency.USD.string,
                    amount = 50.0,
                )
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `initial route amount overrides buy default`() = runTest(testDispatcher) {
        val viewModel = createViewModel(initialAmount = 10.0)

        try {
            runCurrent()

            assertEquals("10", viewModel.amount.value)
            coVerify(exactly = 1) {
                getBuyQuotes(
                    walletId = walletId,
                    asset = asset,
                    type = FiatQuoteType.Buy,
                    fiatCurrency = Currency.USD.string,
                    amount = 10.0,
                )
            }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `fiat type picker requires enabled metadata and available balance`() = runTest(testDispatcher) {
        assetDataFlow.value = assetData(price = 100.0, isSellEnabled = false, available = OneBitcoin)
        val viewModel = createViewModel()

        try {
            runCurrent()
            assertFalse(viewModel.showFiatTypePicker.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = true, available = "0")
            runCurrent()
            assertFalse(viewModel.showFiatTypePicker.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = true, available = OneBitcoin)
            runCurrent()
            assertTrue(viewModel.showFiatTypePicker.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `sell type is only selected when sell is available`() = runTest(testDispatcher) {
        assetDataFlow.value = assetData(price = 100.0, isSellEnabled = false, available = "1")
        val viewModel = createViewModel()

        try {
            runCurrent()
            viewModel.setType(FiatQuoteType.Sell)
            assertEquals(FiatQuoteType.Buy, viewModel.type.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = true, available = "1")
            runCurrent()
            viewModel.setType(FiatQuoteType.Sell)
            assertEquals(FiatQuoteType.Sell, viewModel.type.value)

            assetDataFlow.value = assetData(price = 100.0, isSellEnabled = false, available = OneBitcoin)
            runCurrent()
            assertEquals(FiatQuoteType.Buy, viewModel.type.value)
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

    private fun createViewModel(initialAmount: Double? = null): FiatViewModel {
        val arguments = mutableMapOf<String, Any>(
            RouteArgument.AssetId.key to asset.id.toIdentifier(),
        )
        initialAmount?.let { arguments[RouteArgument.FiatAmount.key] = it }
        return FiatViewModel(
            getBuyQuotes = getBuyQuotes,
            getBuyQuoteUrl = getBuyQuoteUrl,
            getBuyAssetInfo = getBuyAssetInfo,
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
    }
}
