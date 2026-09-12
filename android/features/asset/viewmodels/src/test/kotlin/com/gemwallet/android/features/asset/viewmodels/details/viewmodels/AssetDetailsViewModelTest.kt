package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockPriceAlert
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.PriceAlert
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemAssetDetailsInput
import uniffi.gemstone.GemAssetDetailsServiceInterface
import uniffi.gemstone.GemAssetDetailsState
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemSwapPairSuggestion

@OptIn(ExperimentalCoroutinesApi::class)
class AssetDetailsViewModelTest {
    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetSolanaUSDC()
    private val viewModels = mutableListOf<ViewModel>()

    private val chainAssetInfoFlow = MutableStateFlow<ChainAssetInfo?>(
        ChainAssetInfo(assetInfo = mockAssetInfo(asset), feeAssetInfo = mockAssetInfo(asset)),
    )
    private val sessionFlow = MutableStateFlow<Session?>(mockSession())
    private val banners = MutableSharedFlow<List<Banner>>(replay = 1)
    private val priceAlerts = MutableSharedFlow<List<PriceAlert>>(replay = 1)

    private val getChainAssetInfo = mockk<GetChainAssetInfo>(relaxed = true)
    private val getWalletAssets = mockk<GetWalletAssets>(relaxed = true) {
        every { this@mockk.invoke() } returns MutableStateFlow(emptyList())
    }
    private val getSession = mockk<GetSession>(relaxed = true)
    private val getTransactions = mockk<GetTransactions>(relaxed = true)
    private val getActiveBanners = mockk<GetActiveBanners>(relaxed = true)
    private val getPriceAlerts = mockk<GetPriceAlerts>(relaxed = true)
    private val service = mockk<GemAssetDetailsServiceInterface>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getChainAssetInfo(asset.id) } returns chainAssetInfoFlow
        every { getSession() } returns sessionFlow
        every { getTransactions.getTransactions(any()) } returns MutableStateFlow(emptyList())
        every { getActiveBanners(any(), any()) } returns banners
        every { getPriceAlerts.assetPriceAlerts(asset.id) } returns priceAlerts
        every { service.details(any()) } answers { details(firstArg()) }
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

        priceAlerts.emit(listOf(mockPriceAlert(assetId = asset.id)))
        val uiModel = viewModel.uiModel.first { it != null }!!

        assertEquals(1u, uiModel.detailsState.priceAlertsCount)
        assertEquals(true, uiModel.detailsState.priceAlertEnabled)
    }

    private fun details(input: GemAssetDetailsInput) = GemAssetDetails(
        title = input.asset.name,
        state = GemAssetDetailsState(
            isViewOnly = false,
            headerActions = GemHeaderActions.Buttons(emptyList()),
            showsBanners = input.bannerEvents.isNotEmpty(),
            showsManage = false,
            showsResources = false,
            showsPriceAlerts = input.priceAlerts.isNotEmpty(),
            priceAlertsCount = input.priceAlerts.size.toUInt(),
            priceAlertEnabled = input.priceAlerts.isNotEmpty(),
            showsEarn = false,
            emptyTransactionsAction = null,
        ),
        explorerName = "Explorer",
        addressLink = null,
        tokenLink = null,
        verificationStatus = null,
        networkDestination = null,
        shareUrl = "",
        swapPair = GemSwapPairSuggestion(asset.id.toIdentifier(), null),
    )

    private fun createViewModel(): AssetDetailsViewModel = AssetDetailsViewModel(
        getSession = getSession,
        savedStateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
        getChainAssetInfo = getChainAssetInfo,
        getWalletAssets = getWalletAssets,
        getTransactions = getTransactions,
        assetDetailsService = service,
        getActiveBanners = getActiveBanners,
        getPriceAlerts = getPriceAlerts,
        assetInfoUIModelFactory = AssetInfoUIModelFactory(),
    ).also(viewModels::add)
}
