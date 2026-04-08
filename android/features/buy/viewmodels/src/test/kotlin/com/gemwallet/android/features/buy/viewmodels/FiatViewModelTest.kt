package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockFiatQuote
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
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
    private val sessionFlow = MutableStateFlow<Session?>(Session(wallet, Currency.USD))
    private val assetInfoFlow = MutableStateFlow<AssetInfo?>(assetInfo(price = 100.0))

    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        every { session() } returns sessionFlow
    }
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true) {
        every { getTokenInfo(asset.id) } returns assetInfoFlow
    }
    private val buyRepository = mockk<BuyRepository>(relaxed = true) {
        coEvery {
            getQuotes(
                walletId = wallet.id,
                asset = asset,
                type = any(),
                fiatCurrency = Currency.USD.string,
                amount = 50.0,
            )
        } returns listOf(mockFiatQuote())
    }

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
        createViewModel()

        advanceUntilIdle()

        assetInfoFlow.value = assetInfo(price = 125.0)
        advanceUntilIdle()

        coVerify(exactly = 1) {
            buyRepository.getQuotes(
                walletId = wallet.id,
                asset = asset,
                type = FiatQuoteType.Buy,
                fiatCurrency = Currency.USD.string,
                amount = 50.0,
            )
        }
    }

    @Test
    fun `buy quote loads when asset info becomes available after init`() = runTest(testDispatcher) {
        assetInfoFlow.value = null

        createViewModel()
        advanceUntilIdle()

        assetInfoFlow.value = assetInfo(price = 100.0)
        advanceUntilIdle()

        coVerify(exactly = 1) {
            buyRepository.getQuotes(
                walletId = wallet.id,
                asset = asset,
                type = FiatQuoteType.Buy,
                fiatCurrency = Currency.USD.string,
                amount = 50.0,
            )
        }
    }

    private fun createViewModel() = FiatViewModel(
        sessionRepository = sessionRepository,
        assetsRepository = assetsRepository,
        buyRepository = buyRepository,
        savedStateHandle = SavedStateHandle(
            mapOf("assetId" to asset.id.toIdentifier())
        ),
    )

    private fun assetInfo(price: Double) = mockAssetInfo(asset = asset, walletId = wallet.id).copy(
        price = mockAssetPriceInfo(price = price),
    )
}
