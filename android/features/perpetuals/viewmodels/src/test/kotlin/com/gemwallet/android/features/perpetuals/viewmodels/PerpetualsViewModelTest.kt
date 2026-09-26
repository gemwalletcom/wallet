package com.gemwallet.android.features.perpetuals.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.PerpetualPositionsQuery
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalance
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalanceQuery
import com.gemwallet.android.data.services.store.queries.PerpetualsQuery
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualData
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.WalletId
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
import java.util.concurrent.atomic.AtomicInteger

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private var model: PerpetualsViewModel? = null

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
        val collections = AtomicInteger()
        val pinned = mockPerpetualData(perpetual = mockPerpetual().copy(id = PerpetualId(PerpetualProvider.Hypercore, "BTC"))).copy(metadata = PerpetualMetadata(isPinned = true))
        val unpinned = mockPerpetualData(perpetual = mockPerpetual().copy(id = PerpetualId(PerpetualProvider.Hypercore, "ETH")))
        val markets = flow {
            collections.incrementAndGet()
            emit(listOf(unpinned, pinned))
        }

        val subject = viewModel(mockk(relaxed = true), perpetuals = markets)
        advanceUntilIdle()

        assertEquals(1, collections.get())
        assertEquals(listOf(pinned.perpetual.id), subject.pinnedPerpetuals.value.map { it.id })
        assertEquals(listOf(unpinned.perpetual.id), subject.unpinnedPerpetuals.value.map { it.id })
    }

    private fun viewModel(
        service: GemPerpetualServiceInterface,
        positions: List<PerpetualPositionData> = emptyList(),
        balance: PerpetualBalance? = null,
        walletType: WalletType = WalletType.Multicoin,
        perpetuals: Flow<List<PerpetualData>> = flowOf(emptyList()),
    ): PerpetualsViewModel {
        val perpetualsQuery = mockk<PerpetualsQuery> {
            every { this@mockk(any(), any(), any()) } returns perpetuals
        }
        val perpetualPositionsQuery = mockk<PerpetualPositionsQuery> {
            every { this@mockk(any<WalletId>()) } returns flowOf(positions)
        }
        val perpetualWalletBalanceQuery = mockk<PerpetualWalletBalanceQuery> {
            every { this@mockk(any(), any()) } returns flowOf(balance?.let { PerpetualWalletBalance(balance = it, price = 1.0) })
        }
        val recentActivityQuery = mockk<RecentActivityQuery> {
            every { this@mockk(any(), any(), any(), any()) } returns flowOf(emptyList())
        }
        val perpetualObserver = mockk<PerpetualObserver>()
        val getSession = mockk<GetSession>()
        every { getSession() } returns MutableStateFlow(mockSession(wallet = mockWallet(type = walletType)))

        return PerpetualsViewModel(
            perpetualsQuery = perpetualsQuery,
            perpetualPositionsQuery = perpetualPositionsQuery,
            perpetualWalletBalanceQuery = perpetualWalletBalanceQuery,
            getSession = getSession,
            recentActivityQuery = recentActivityQuery,
            service = service,
            perpetualObserver = perpetualObserver,
            ioDispatcher = dispatcher,
            observeRefreshInterval = mockk(relaxed = true),
        ).also { model = it }
    }

    @Test
    fun `a position is found by its perpetual symbol like on iOS`() = runTest(dispatcher) {
        val position = mockPerpetualPositionData(
            perpetual = mockPerpetual(price = 100.0).copy(id = PerpetualId(PerpetualProvider.Hypercore, "BTC-USD")),
            asset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC"),
        )
        val viewModel = viewModel(mockk(relaxed = true), positions = listOf(position))
        viewModel.setQuery("btc-usd")
        assertEquals(listOf(position.perpetual.id), viewModel.positions.first { it.isNotEmpty() }.map { it.perpetualId })
        viewModel.setQuery("zzz")
        assertEquals(emptyList<PerpetualId>(), viewModel.positions.first { it.isEmpty() }.map { it.perpetualId })
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
