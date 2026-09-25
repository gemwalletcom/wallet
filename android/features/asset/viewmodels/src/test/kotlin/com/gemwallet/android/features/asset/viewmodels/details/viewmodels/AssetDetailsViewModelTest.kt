package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.data.services.store.queries.BannersQuery
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockChainAssetInfo
import com.gemwallet.android.testkit.mockGemAssetDetails
import com.gemwallet.android.testkit.mockGemAssetDetailsState
import com.gemwallet.android.testkit.mockPriceAlert
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.PriceAlertData
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PriceAlert
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetDetailsInput
import uniffi.gemstone.GemAssetDetailsServiceInterface
import uniffi.gemstone.GemAssetRefresh
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertToggle

@OptIn(ExperimentalCoroutinesApi::class)
class AssetDetailsViewModelTest {
    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)
    private val viewModels = mutableListOf<ViewModel>()

    private val chainAssetInfoFlow = MutableStateFlow<ChainAssetInfo?>(
        mockChainAssetInfo(mockAssetInfo(asset)),
    )
    private val sessionFlow = MutableStateFlow<Session?>(mockSession())
    private val banners = MutableSharedFlow<List<Banner>>(replay = 1)
    private val priceAlerts = MutableSharedFlow<List<PriceAlertData>>(replay = 1)

    private val getChainAssetInfo = mockk<GetChainAssetInfo>(relaxed = true)
    private val getWalletAssets = mockk<GetWalletAssets>(relaxed = true) {
        every { this@mockk.invoke() } returns MutableStateFlow(emptyList())
    }
    private val getSession = mockk<GetSession>(relaxed = true)
    private val getTransactions = mockk<GetTransactions>(relaxed = true)
    private val bannersQuery = mockk<BannersQuery>(relaxed = true)
    private val priceAlertsQuery = mockk<PriceAlertsQuery>(relaxed = true)
    private val service = mockk<GemAssetDetailsServiceInterface>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getChainAssetInfo(asset.id) } returns chainAssetInfoFlow
        every { getSession() } returns sessionFlow
        every { getTransactions.getTransactions(any()) } returns MutableStateFlow(emptyList())
        every { getTransactions.stored(any()) } returns emptyList()
        every { bannersQuery(mockSession().wallet.id.id, asset.id) } returns banners
        every { priceAlertsQuery(asset.id) } returns priceAlerts
        every { service.details(any()) } answers {
            val input = firstArg<GemAssetDetailsInput>()
            mockGemAssetDetails(asset, mockGemAssetDetailsState(showsBanners = input.banners.isNotEmpty(), priceAlertsCount = input.priceAlerts.size))
        }
    }

    @After
    fun tearDown() {
        viewModels.forEach { it.viewModelScope.cancel() }
        viewModels.clear()
        Dispatchers.resetMain()
    }

    @Test
    fun `the scene waits for the banners and the alerts instead of drawing without them`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        advanceUntilIdle()

        assertNull(viewModel.uiModel.value)

        banners.emit(emptyList())
        advanceUntilIdle()

        assertNull(viewModel.uiModel.value)

        priceAlerts.emit(listOf(PriceAlertData(asset = asset, priceAlert = mockPriceAlert(assetId = asset.id), rankScore = 0)))
        val uiModel = viewModel.uiModel.first { it != null }!!

        assertEquals(GemPriceAlertToggle.ENABLED, uiModel.details.state.priceAlert)
    }

    @Test
    fun `the sync the screen starts while it is built reads only state that is already set`() = runTest(testDispatcher) {
        val immediate = UnconfinedTestDispatcher(testScheduler)
        Dispatchers.setMain(immediate)
        coEvery { service.refresh(any(), any()) } returns GemAssetRefresh(transactions = GemLoadState.Data, failures = emptyList())

        createViewModel(ioDispatcher = immediate)

        coVerify { service.refresh(asset.id.toIdentifier(), false) }
    }

    private fun createViewModel(ioDispatcher: CoroutineDispatcher = testDispatcher): AssetDetailsViewModel = AssetDetailsViewModel(
        getSession = getSession,
        savedStateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
        getChainAssetInfo = getChainAssetInfo,
        getWalletAssets = getWalletAssets,
        getTransactions = getTransactions,
        assetDetailsService = service,
        bannersQuery = bannersQuery,
        priceAlertsQuery = priceAlertsQuery,
        assetInfoUIModelFactory = AssetInfoUIModelFactory(mockk<Context> { every { getString(any()) } answers { firstArg<Int>().toString() } }),
        userConfig = mockk(relaxed = true),
        ioDispatcher = ioDispatcher,
        connectionStatusObserver = mockk(relaxed = true),
        context = mockk(relaxed = true),
    ).also(viewModels::add)
}
