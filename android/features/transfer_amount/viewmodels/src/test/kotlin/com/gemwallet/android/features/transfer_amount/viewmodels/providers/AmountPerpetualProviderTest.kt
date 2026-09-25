package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAmountParamsPerpetual
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockPerpetualData
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.PerpetualDirection
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemLeverageSelection
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualAutoclose
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemPickerOption

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProviderTest {

    @Test
    fun `title carries the direction`() {
        val provider = makeProvider(direction = PerpetualDirection.Short)
        val title = provider.request.value?.amountType()?.title() as GemAmountTitle.PerpetualOpen
        assertEquals(PerpetualDirection.Short.toGem(), title.direction)
    }

    @Test
    fun `editing then clearing a trigger keeps it cleared when no default is set`() = runTest {
        val provider = makeProvider(scope = backgroundScope)
        provider.setTakeProfit("65000")
        provider.setStopLoss("55000")
        provider.setTakeProfit("")
        provider.setStopLoss(null)
        advanceUntilIdle()
        assertNull(provider.takeProfit.value)
        assertNull(provider.stopLoss.value)
    }

    @Test
    fun `a leverage change refreshes untouched defaults and keeps an edited price`() = runTest {
        val provider = makeProvider(autoclose = { leverage -> GemPerpetualAutoclose(takeProfit = "${60000 + leverage.toInt()}", stopLoss = "${40000 - leverage.toInt()}") })
        val defaultStopLoss = provider.stopLoss.value

        provider.setTakeProfit("99999")
        provider.setLeverage(10u)

        assertEquals("99999", provider.takeProfit.value)
        assertTrue(provider.stopLoss.value != null && provider.stopLoss.value != defaultStopLoss)
    }

    @Test
    fun `showsAutoclose is true for Open and false for Reduce`() {
        assertTrue(makeProvider().showsAutoclose)
        val reduce = makeProvider(positionAction = GemPerpetualPositionAction.Reduce(mockGemPerpetualTransferData(direction = PerpetualDirection.Long), mockPerpetualPosition().toGem()))
        assertFalse(reduce.showsAutoclose)
    }

    private fun makeProvider(
        direction: PerpetualDirection = PerpetualDirection.Long,
        positionAction: GemPerpetualPositionAction = GemPerpetualPositionAction.Open(mockGemPerpetualTransferData(direction = direction)),
        scope: CoroutineScope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob()),
        autoclose: (UByte) -> GemPerpetualAutoclose = { GemPerpetualAutoclose(takeProfit = null, stopLoss = null) },
    ): AmountPerpetualProvider {
        val getAssetInfo = mockk<GetAssetInfo>(relaxed = true) {
            every { this@mockk.invoke(any()) } returns flowOf(null)
        }
        val service = mockk<GemAmountServiceInterface> {
            every { perpetualLeverageSelection(any()) } returns GemLeverageSelection(
                options = listOf(
                    GemPickerOption(value = 5u, label = GemLocalizedText.Text("5x")),
                    GemPickerOption(value = 10u, label = GemLocalizedText.Text("10x")),
                ),
                selected = GemPickerOption(value = 5u, label = GemLocalizedText.Text("5x")),
            )
            every { perpetualAutoclose(any(), any(), any()) } answers { autoclose(secondArg<Byte>().toUByte()) }
            every { perpetualAutocloseRow(any(), any()) } returns GemListRow.Lines(GemListRowTitle.AUTO_CLOSE, emptyList(), null)
        }
        val perpetualAggregate = mockPerpetualData()
        val getPerpetual = mockk<GetPerpetual>(relaxed = true) {
            every { getPerpetual(any()) } returns flowOf(perpetualAggregate)
        }
        return AmountPerpetualProvider(
            params = mockAmountParamsPerpetual(positionAction),
            context = mockk(relaxed = true),
            service = service,
            getAssetInfo = getAssetInfo,
            getPerpetual = getPerpetual,
            scope = scope,
        )
    }
}
