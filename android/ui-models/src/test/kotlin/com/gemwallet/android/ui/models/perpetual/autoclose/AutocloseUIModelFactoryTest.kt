package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.domains.perpetual.autoclose.AutocloseError
import com.gemwallet.android.testkit.mockAutocloseField
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class AutocloseUIModelFactoryTest {

    @Test
    fun percentSuggestionsScaleWithLeverage() {
        assertEquals(listOf(5, 10, 15), model(leverage = 1u).takeProfit.percentSuggestions)
        assertEquals(listOf(10, 15, 25), model(leverage = 5u).takeProfit.percentSuggestions)
        assertEquals(listOf(15, 25, 50), model(leverage = 10u).takeProfit.percentSuggestions)
        assertEquals(listOf(25, 50, 100), model(leverage = 20u).takeProfit.percentSuggestions)
    }

    @Test
    fun pnlSuppressedWhenFieldHasError() {
        val invalid = mockAutocloseField(TpslType.TakeProfit, price = 50.0, error = AutocloseError.TriggerMustBeHigher)
        val model = AutocloseUIModelFactory.create(
            position = mockPerpetualPositionData(
                position = mockPerpetualPosition(direction = PerpetualDirection.Long, entryPrice = 100.0, leverage = 5u),
            ),
            takeProfit = invalid,
            stopLoss = mockAutocloseField(TpslType.StopLoss),
            confirmEnabled = false,
        )
        assertEquals("-", model.takeProfit.pnlText)
    }

    @Test
    fun errorSuppressedUntilShowErrorsSet() {
        val invalidTakeProfit = mockAutocloseField(TpslType.TakeProfit, price = 50.0, error = AutocloseError.TriggerMustBeHigher)
        val hidden = AutocloseUIModelFactory.create(
            position = mockPerpetualPositionData(
                position = mockPerpetualPosition(direction = PerpetualDirection.Long, entryPrice = 100.0, leverage = 5u),
            ),
            takeProfit = invalidTakeProfit,
            stopLoss = mockAutocloseField(TpslType.StopLoss),
            confirmEnabled = false,
            showErrors = false,
        )
        val shown = AutocloseUIModelFactory.create(
            position = mockPerpetualPositionData(
                position = mockPerpetualPosition(direction = PerpetualDirection.Long, entryPrice = 100.0, leverage = 5u),
            ),
            takeProfit = invalidTakeProfit,
            stopLoss = mockAutocloseField(TpslType.StopLoss),
            confirmEnabled = false,
            showErrors = true,
        )
        assertFalse(hidden.takeProfit.showError)
        assertEquals(AutocloseError.TriggerMustBeHigher, shown.takeProfit.error)
    }

    private fun model(leverage: UByte) = AutocloseUIModelFactory.create(
        position = mockPerpetualPositionData(position = mockPerpetualPosition(leverage = leverage)),
        takeProfit = mockAutocloseField(TpslType.TakeProfit),
        stopLoss = mockAutocloseField(TpslType.StopLoss),
        confirmEnabled = false,
    )
}
