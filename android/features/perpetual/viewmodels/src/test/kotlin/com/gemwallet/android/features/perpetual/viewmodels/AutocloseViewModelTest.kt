package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositionByAsset
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.TpslType
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain

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

    private fun positionData(direction: PerpetualDirection = PerpetualDirection.Long) = mockPerpetualPositionData(
        perpetual = mockPerpetual(price = 100.0),
        asset = asset,
        position = mockPerpetualPosition(
            assetId = asset.id,
            direction = direction,
            entryPrice = 100.0,
            leverage = 5u,
        ),
    )

    private fun viewModel(position: PerpetualPositionData? = positionData()): AutocloseViewModel {
        val session: GetSession = mockk {
            every { this@mockk.invoke() } returns MutableStateFlow(mockSession())
        }
        val byAsset: GetPerpetualPositionByAsset = mockk {
            every { this@mockk.invoke(any(), any()) } returns flowOf(position)
        }
        return AutocloseViewModel(
            byAsset,
            session,
            SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
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
}
