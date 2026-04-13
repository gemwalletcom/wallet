package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockFiatQuote
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
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
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class FiatViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset()
    private val wallet = mockWallet(id = "wallet-id")
    private val walletId = WalletId(wallet.id)
    private val assetDataFlow = MutableStateFlow<AssetData?>(assetData(price = 100.0))

    private val getBuyAssetInfo = object : GetBuyAssetInfo {
        override fun invoke(assetId: AssetId): Flow<AssetData?> = assetDataFlow
    }
    private val getBuyQuotes = mockk<GetBuyQuotes>(relaxed = true) {
        coEvery {
            invoke(
                walletId = walletId,
                asset = asset,
                type = any(),
                fiatCurrency = Currency.USD.string,
                amount = 50.0,
            )
        } returns listOf(mockFiatQuote())
    }
    private val getBuyQuoteUrl = mockk<GetBuyQuoteUrl>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
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

    private fun createViewModel() = FiatViewModel(
        getBuyQuotes = getBuyQuotes,
        getBuyQuoteUrl = getBuyQuoteUrl,
        getBuyAssetInfo = getBuyAssetInfo,
        savedStateHandle = SavedStateHandle(
            mapOf("assetId" to asset.id.toIdentifier())
        ),
    )

    private fun assetData(price: Double) = mockAssetData(
        asset = asset,
        wallet = wallet,
    ).copy(price = mockAssetPriceInfo(price = price))
}
