package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.perpetual.cases.PerpetualSections
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.WalletType
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemHeaderButtonKind
import uniffi.gemstone.GemMarketsRefreshTrigger
import uniffi.gemstone.GemPerpetualBalanceHeader
import uniffi.gemstone.GemPerpetualServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualMarketViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private var model: PerpetualMarketViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        model?.viewModelScope?.cancel()
        model = null
        Dispatchers.resetMain()
    }

    @Test
    fun `pull to refresh asks core for a user requested markets sync`() = runTest(dispatcher) {
        val trigger = CompletableDeferred<GemMarketsRefreshTrigger>()
        val service = mockk<GemPerpetualServiceInterface>()
        coEvery { service.refresh(any()) } answers {
            trigger.complete(firstArg())
            emptyList()
        }

        viewModel(service).onRefresh()

        assertEquals(GemMarketsRefreshTrigger.USER_REQUESTED, trigger.await())
    }

    @Test
    fun `opening the screen asks core for a scheduled refresh, not positions alone`() = runTest(dispatcher) {
        val trigger = CompletableDeferred<GemMarketsRefreshTrigger>()
        val service = mockk<GemPerpetualServiceInterface>()
        coEvery { service.refresh(any()) } answers {
            trigger.complete(firstArg())
            emptyList()
        }

        viewModel(service).refreshMarkets()

        assertEquals(GemMarketsRefreshTrigger.SCHEDULED, trigger.await())
    }

    @Test
    fun `pinned and unpinned markets share one market observation`() = runTest(dispatcher) {
        var collections = 0
        val pinned = mockk<PerpetualDataAggregate>()
        val unpinned = mockk<PerpetualDataAggregate>()
        val markets = flow {
            collections++
            emit(PerpetualSections(pinned = listOf(pinned), markets = listOf(unpinned)))
        }

        val subject = viewModel(mockk(relaxed = true), perpetuals = markets)
        advanceUntilIdle()

        assertEquals(1, collections)
        assertEquals(listOf(pinned), subject.pinnedPerpetuals.value)
        assertEquals(listOf(unpinned), subject.unpinnedPerpetuals.value)
    }

    private fun viewModel(
        service: GemPerpetualServiceInterface,
        positions: List<PerpetualPositionDataAggregate> = emptyList(),
        balance: PerpetualBalance? = null,
        walletType: WalletType = WalletType.Multicoin,
        perpetuals: Flow<PerpetualSections> = flowOf(PerpetualSections()),
    ): PerpetualMarketViewModel {
        val getPerpetuals = mockk<GetPerpetuals>()
        every { getPerpetuals.getPerpetualSections(any()) } returns perpetuals
        val getPositions = mockk<GetPerpetualPositions>()
        every { getPositions.getPerpetualPositions() } returns flowOf(positions)
        val getBalance = mockk<GetPerpetualBalance>()
        every { getBalance.getBalance() } returns flowOf(balance)
        val recentAssetsService = mockk<RecentAssetsService>()
        every { recentAssetsService.getRecentAssets(any()) } returns flowOf(emptyList())
        val perpetualObserver = mockk<PerpetualObserver>()
        val getSession = mockk<GetSession>()
        every { getSession() } returns MutableStateFlow(mockSession(wallet = mockWallet(type = walletType)))

        return PerpetualMarketViewModel(
            getPerpetuals = getPerpetuals,
            getPositions = getPositions,
            getBalance = getBalance,
            getSession = getSession,
            recentAssetsService = recentAssetsService,
            service = service,
            perpetualObserver = perpetualObserver,
            ioDispatcher = dispatcher,
            connectionStatusObserver = mockk(relaxed = true),
        ).also { model = it }
    }

    @Test
    fun `a position is found by its perpetual symbol like on iOS`() = runTest(dispatcher) {
        val position = mockk<PerpetualPositionDataAggregate>(relaxed = true) {
            every { title } returns "Bitcoin"
            every { perpetualId } returns PerpetualId(PerpetualProvider.Hypercore, "BTC-USD")
            every { asset } returns mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC")
        }
        val viewModel = viewModel(mockk(relaxed = true), positions = listOf(position))
        viewModel.setQuery("btc-usd")
        assertEquals(listOf(position), viewModel.positions.first { it.isNotEmpty() })
        viewModel.setQuery("zzz")
        assertEquals(emptyList<PerpetualPositionDataAggregate>(), viewModel.positions.first { it.isEmpty() })
    }

    @Test
    fun `the header comes from core for the session wallet and the stored balance`() = runTest(dispatcher) {
        val leveraged = PerpetualBalance(available = 50.0, reserved = 50.0, withdrawable = 0.0)

        val funded = viewModel(mockk(relaxed = true), balance = leveraged)
        val watching = viewModel(mockk(relaxed = true), balance = leveraged, walletType = WalletType.View)
        advanceUntilIdle()

        assertEquals(false, requireNotNull(funded.balanceHeader.value).withdrawEnabled())
        assertEquals(GemHeaderActions.WatchOnly, watching.balanceHeader.value?.actions)
    }

    private fun GemPerpetualBalanceHeader.withdrawEnabled(): Boolean? = (actions as? GemHeaderActions.Buttons)?.buttons?.first { it.kind == GemHeaderButtonKind.WITHDRAW }?.isEnabled
}
