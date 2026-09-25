package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.PerpetualPositionsQuery
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualData
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.TpslType
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class AutocloseViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<AutocloseViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val asset = mockAsset()

    private val positions: PerpetualPositionsQuery = mockk()

    private fun viewModel(
        position: PerpetualPositionData? = mockPerpetualPositionData(
            perpetual = mockPerpetual(price = 100.0),
            position = mockPerpetualPosition(size = 10.0, sizeValue = 1000.0, leverage = 5u, entryPrice = 100.0, liquidationPrice = 50.0, marginType = PerpetualMarginType.Cross, direction = PerpetualDirection.Long, marginAmount = 200.0),
        ),
        market: PerpetualData? = mockPerpetualData(),
    ): AutocloseViewModel {
        val session: GetSession = mockk {
            every { this@mockk.invoke() } returns MutableStateFlow(mockSession())
        }
        val perpetual: PerpetualQuery = mockk {
            every { this@mockk(asset.id) } returns flowOf(market)
        }
        every { positions(mockSession().wallet.id, any<PerpetualId>()) } returns flowOf(position)
        return AutocloseViewModel(
            perpetual,
            positions,
            session,
            SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
            dispatcher,
            mockk(relaxed = true),
        ).also { models.add(it) }
    }

    @Test
    fun `a take profit percent becomes a price above the entry for a long`() = runTest(dispatcher) {
        val model = viewModel()
        model.position.first { it != null }

        model.onPercentSelected(TpslType.TakeProfit, 50)

        val takeProfit = model.takeProfitText.first { it.isNotEmpty() }
        assertTrue("expected a target above the entry price, got $takeProfit", takeProfit.toDouble() > 100.0)
    }

    @Test
    fun `a stop loss percent becomes a price below the entry for a long`() = runTest(dispatcher) {
        val model = viewModel()
        model.position.first { it != null }

        model.onPercentSelected(TpslType.StopLoss, 50)

        val stopLoss = model.stopLossText.first { it.isNotEmpty() }
        assertTrue("expected a target below the entry price, got $stopLoss", stopLoss.toDouble() < 100.0)
    }

    @Test
    fun `confirming with nothing entered emits no transfer`() = runTest(dispatcher) {
        val model = viewModel()
        model.position.first { it != null }

        model.onConfirm()

        assertEquals(0, model.confirmRequests.replayCache.size)
    }

    @Test
    fun `an entered price is sanitized before it reaches Core`() = runTest(dispatcher) {
        val model = viewModel()
        model.position.first { it != null }

        model.onTakeProfitChanged("12a3.4b5")

        assertEquals("123.45", model.takeProfitText.first { it.isNotEmpty() })
    }

    @Test
    fun `the view state carries each field's estimate once a price is entered`() = runTest(dispatcher) {
        val model = viewModel()
        model.position.first { it != null }

        model.onTakeProfitChanged("150")

        val state = model.viewState.first { it?.takeProfit?.estimate != null }
        assertNull(state?.stopLoss?.estimate)
    }

    @Test
    fun `a transfer Core refuses reports its error instead of doing nothing`() = runTest(dispatcher) {
        val model = viewModel(
            mockPerpetualPositionData(
                perpetual = mockPerpetual(identifier = "BTC", price = 100.0),
                position = mockPerpetualPosition(
                    size = 10.0,
                    sizeValue = 1000.0,
                    leverage = 5u,
                    entryPrice = 100.0,
                    liquidationPrice = 50.0,
                    marginType = PerpetualMarginType.Cross,
                    direction = PerpetualDirection.Long,
                    marginAmount = 200.0,
                ),
            ),
        )
        model.position.first { it != null }
        val errors = mutableListOf<String>()
        backgroundScope.launch { model.errors.collect { errors.add(it) } }

        model.onTakeProfitChanged("150")
        model.viewState.first { it?.takeProfit?.estimate != null }
        model.onConfirm()
        advanceUntilIdle()

        assertEquals(1, errors.size)
        assertEquals(0, model.confirmRequests.replayCache.size)
    }

    @Test
    fun `the position is read for the market of the asset in the session wallet`() = runTest(dispatcher) {
        val market = mockPerpetualData()
        val model = viewModel(market = market)

        model.position.first { it != null }

        verify(exactly = 1) { positions(mockSession().wallet.id, market.perpetual.id) }
    }

    @Test
    fun `an asset without a stored market has no position`() = runTest(dispatcher) {
        val model = viewModel(market = null)
        advanceUntilIdle()

        assertNull(model.position.value)
        verify(exactly = 0) { positions(any(), any<PerpetualId>()) }
    }
}
