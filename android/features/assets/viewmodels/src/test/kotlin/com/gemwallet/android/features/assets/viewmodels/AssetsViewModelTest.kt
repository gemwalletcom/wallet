package com.gemwallet.android.features.assets.viewmodels

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetHideBalancesState
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.session.cases.GetSession
import uniffi.gemstone.GemWalletHomeServiceInterface
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.coVerify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.first
import org.junit.Assert.assertFalse
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class AssetsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val activeAssetsFlow = MutableStateFlow(
        listOf(
            assetAggregate(chain = Chain.Solana, symbol = "SOL", pinned = true),
            assetAggregate(chain = Chain.Ethereum, symbol = "ETH", pinned = false),
        )
    )

    private val service = mockk<GemWalletHomeServiceInterface>(relaxed = true)
    private val getActiveAssetsInfo = object : GetActiveAssetsInfo {
        override fun getAssetsInfo(hideBalance: Boolean): Flow<List<AssetInfoDataAggregate>> = activeAssetsFlow
    }
    private val getWalletSummary = mockk<GetWalletSummary>(relaxed = true) {
        every { getWalletSummary() } returns flowOf(null)
    }
    private val getHideBalancesState = object : GetHideBalancesState {
        override fun invoke(): Flow<Boolean> = flowOf(false)
    }
    private val session = MutableStateFlow<Session?>(null)
    private val getSession = object : GetSession {
        override fun invoke(): StateFlow<Session?> = session
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
    fun `pinned and unpinned assets replay current wallet assets`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        advanceUntilIdle()

        assertEquals(listOf(activeAssetsFlow.value[0]), viewModel.pinnedAssets.value)
        assertEquals(listOf(activeAssetsFlow.value[1]), viewModel.unpinnedAssets.value)
    }

    @Test
    fun `the loading row follows core's first load answer around the refresh`() = runTest(testDispatcher) {
        val refreshStarted = CompletableDeferred<Unit>()
        val refreshGate = CompletableDeferred<Unit>()
        every { service.showsInitialLoading() } returns true
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

    private fun createViewModel() = AssetsViewModel(
        service = service,
        getActiveAssetsInfo = getActiveAssetsInfo,
        getWalletSummary = getWalletSummary,
        getHideBalancesState = getHideBalancesState,
        getSession = getSession,
        userConfig = mockk(relaxed = true),
    )

    private fun assetAggregate(
        chain: Chain,
        symbol: String,
        pinned: Boolean,
    ): AssetInfoDataAggregate {
        val asset = mockAsset(chain = chain, name = symbol, symbol = symbol)
        return AssetInfoDataAggregate(
            id = asset.id,
            asset = asset,
            title = asset.name,
            balance = "1.0 $symbol",
            balanceEquivalent = "$1.00",
            isZeroBalance = false,
            price = null,
            pinned = pinned,
            balanceEnabled = true,
            accountAddress = "address-$symbol",
        )
    }
}
