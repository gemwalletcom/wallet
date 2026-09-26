package com.gemwallet.android.features.wallet_tab.viewmodels

import android.util.Log
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfoDataAggregate
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletHomeServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class AssetsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val activeAssetsFlow = MutableStateFlow(
        listOf(
            mockAssetInfoDataAggregate(asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9), pinned = true),
            mockAssetInfoDataAggregate(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)),
        ),
    )

    private val service = mockk<GemWalletHomeServiceInterface>(relaxed = true)
    private val getActiveAssetsInfo = object : GetActiveAssetsInfo {
        override fun assetsInfo(): StateFlow<List<AssetInfoDataAggregate>> = activeAssetsFlow
    }
    private val getWalletSummary = mockk<GetWalletSummary>(relaxed = true) {
        every { getWalletSummary() } returns flowOf(null)
    }
    private val session = MutableStateFlow<Session?>(null)
    private val getSession = object : GetSession {
        override fun invoke(): StateFlow<Session?> = session
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        unmockkStatic(Log::class)
    }

    @Test
    fun `pinned and unpinned assets replay current wallet assets`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        assertEquals("the first frame is already grouped", listOf(activeAssetsFlow.value[0]), viewModel.pinnedAssets.value)
        assertEquals(listOf(activeAssetsFlow.value[1]), viewModel.unpinnedAssets.value)

        advanceUntilIdle()

        assertEquals(listOf(activeAssetsFlow.value[0]), viewModel.pinnedAssets.value)
        assertEquals(listOf(activeAssetsFlow.value[1]), viewModel.unpinnedAssets.value)
    }

    @Test
    fun `a pin change moves the asset between the sections in one update`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        advanceUntilIdle()
        val solana = mockAssetInfoDataAggregate(asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9))
        val ethereum = mockAssetInfoDataAggregate(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18), pinned = true)

        activeAssetsFlow.value = listOf(solana, ethereum)
        advanceUntilIdle()

        assertEquals(listOf(ethereum), viewModel.pinnedAssets.value)
        assertEquals(listOf(solana), viewModel.unpinnedAssets.value)
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

    private fun createViewModel() = AssetsViewModel(
        service = service,
        getActiveAssetsInfo = getActiveAssetsInfo,
        getWalletSummary = getWalletSummary,
        getSession = getSession,
        preferences = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
        context = mockk(relaxed = true),
    )
}
