package com.gemwallet.android.features.wallet.viewmodels

import android.util.Log
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletHomeState
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfoDataAggregate
import com.gemwallet.android.testkit.mockGemWalletHomeViewState
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestCoroutineScheduler
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemNftEntry
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.GemWalletHomeViewState

@OptIn(ExperimentalCoroutinesApi::class)
class WalletViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(testDispatcher)

    private val activeAssetsFlow = MutableStateFlow(
        listOf(
            mockAssetInfoDataAggregate(asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9u), pinned = true),
            mockAssetInfoDataAggregate(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18u)),
        ),
    )

    private val service = mockk<GemWalletHomeServiceInterface>(relaxed = true)
    private val getActiveAssetsInfo = object : GetActiveAssetsInfo {
        override fun assetsInfo(): StateFlow<List<AssetInfoDataAggregate>> = activeAssetsFlow
    }
    private val homeState = MutableStateFlow<GemWalletHomeViewState?>(
        mockGemWalletHomeViewState(
            pinnedAssetIds = listOf(mockAssetId(chain = Chain.Solana).toIdentifier()),
            assetIds = listOf(mockAssetId(chain = Chain.Ethereum).toIdentifier()),
        ),
    )
    private val getWalletHomeState = object : GetWalletHomeState {
        override fun walletHomeState(): StateFlow<GemWalletHomeViewState?> = homeState

        override fun walletCollections(): StateFlow<List<GemNftEntry>> = MutableStateFlow(emptyList())
    }
    private val session = MutableStateFlow<Session?>(null)
    private val getSession = object : GetSession {
        override fun invoke(): StateFlow<Session?> = session
    }

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        unmockkStatic(Log::class)
    }

    @Test
    fun `the first frame already shows the home state's sections`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        assertEquals(listOf(activeAssetsFlow.value[0]), viewModel.pinnedAssets.value)
        assertEquals(listOf(activeAssetsFlow.value[1]), viewModel.unpinnedAssets.value)
    }

    @Test
    fun `the sections follow the ids of a new home state`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        advanceUntilIdle()

        homeState.value = mockGemWalletHomeViewState(
            pinnedAssetIds = listOf(mockAssetId(chain = Chain.Ethereum).toIdentifier()),
            assetIds = listOf(mockAssetId(chain = Chain.Solana).toIdentifier()),
        )
        advanceUntilIdle()

        assertEquals(listOf(activeAssetsFlow.value[1]), viewModel.pinnedAssets.value)
        assertEquals(listOf(activeAssetsFlow.value[0]), viewModel.unpinnedAssets.value)
    }

    @Test
    fun `the loading row follows core's first load answer around the refresh`() = runTest(testDispatcher) {
        val refreshStarted = CompletableDeferred<Unit>()
        val refreshGate = CompletableDeferred<Unit>()
        every { service.showsInitialLoading() } returnsMany listOf(true, false)
        coEvery { service.refresh() } coAnswers {
            refreshStarted.complete(Unit)
            refreshGate.await()
        }
        session.value = mockSession(wallet = mockWallet())

        val viewModel = createViewModel()
        advanceUntilIdle()

        refreshStarted.await()
        assertTrue(viewModel.isLoadingAssets.value)
        refreshGate.complete(Unit)
        assertFalse(viewModel.isLoadingAssets.first { !it })
        coVerify(exactly = 1) { service.refresh() }
    }

    @Test
    fun `a wallet core has already loaded refreshes without the loading row`() = runTest(testDispatcher) {
        val refreshStarted = CompletableDeferred<Unit>()
        val refreshGate = CompletableDeferred<Unit>()
        every { service.showsInitialLoading() } returns false
        coEvery { service.refresh() } coAnswers {
            refreshStarted.complete(Unit)
            refreshGate.await()
        }
        session.value = mockSession(wallet = mockWallet())

        val viewModel = createViewModel()
        advanceUntilIdle()

        refreshStarted.await()
        assertFalse(viewModel.isLoadingAssets.value)
        refreshGate.complete(Unit)
    }

    @Test
    fun `the loading row ends while nfts are still loading`() = runTest(testDispatcher) {
        val nftGate = CompletableDeferred<Unit>()
        every { service.showsInitialLoading() } returnsMany listOf(true, false)
        coEvery { service.refreshNfts() } coAnswers { nftGate.await() }
        session.value = mockSession(wallet = mockWallet())

        val viewModel = createViewModel()
        advanceUntilIdle()

        assertFalse(viewModel.isLoadingAssets.value)
        coVerify(exactly = 1) { service.refreshNfts() }
        nftGate.complete(Unit)
    }

    @Test
    fun `a failed discovery keeps the loading row until a later refresh completes it`() = runTest(testDispatcher) {
        var discovered = false
        every { service.showsInitialLoading() } answers { !discovered }
        coEvery { service.refresh() } throws GemServiceException.Gateway("offline")
        session.value = mockSession(wallet = mockWallet())

        val viewModel = createViewModel()
        advanceUntilIdle()

        assertTrue(viewModel.isLoadingAssets.value)

        coEvery { service.refresh() } answers { discovered = true }
        viewModel.onRefresh()
        advanceUntilIdle()

        assertFalse(viewModel.isLoadingAssets.value)
    }

    @Test
    fun `hiding balances writes the preference off the main thread`() = runTest(testDispatcher) {
        val preferences = mockk<ObservablePreferences>(relaxed = true)
        val ioDispatcher = StandardTestDispatcher(TestCoroutineScheduler())
        val viewModel = createViewModel(preferences = preferences, ioDispatcher = ioDispatcher)

        viewModel.hideBalances()
        advanceUntilIdle()

        verify(exactly = 0) { preferences.hideBalances() }

        ioDispatcher.scheduler.advanceUntilIdle()

        verify(exactly = 1) { preferences.hideBalances() }
    }

    private fun createViewModel(preferences: ObservablePreferences = mockk(relaxed = true), ioDispatcher: CoroutineDispatcher = testDispatcher) = WalletViewModel(
        service = service,
        getActiveAssetsInfo = getActiveAssetsInfo,
        getWalletHomeState = getWalletHomeState,
        getSession = getSession,
        preferences = preferences,
        ioDispatcher = ioDispatcher,
        context = mockk(relaxed = true),
    )
}
