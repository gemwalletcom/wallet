package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.testkit.mockAutocloseField
import com.gemwallet.android.testkit.mockAutocloseViewState
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.wallet.core.primitives.TpslType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.AutocloseValidation

class AutocloseUIModelFactoryTest {

    @Test
    fun percentSuggestionsScaleWithLeverage() {
        assertEquals(listOf(5, 10, 15), model(leverage = 1u).takeProfit.percentSuggestions.map { it.value.toInt() })
        assertEquals(listOf(10, 15, 25), model(leverage = 5u).takeProfit.percentSuggestions.map { it.value.toInt() })
        assertEquals(listOf(15, 25, 50), model(leverage = 10u).takeProfit.percentSuggestions.map { it.value.toInt() })
        assertEquals(listOf(25, 50, 100), model(leverage = 20u).takeProfit.percentSuggestions.map { it.value.toInt() })
    }

    @Test
    fun pnlEstimatesEvenWhenTheTriggerIsInvalid() {
        val invalid = mockAutocloseField(TpslType.TakeProfit, price = 50.0, validation = AutocloseValidation.TRIGGER_MUST_BE_HIGHER)
        val model = AutocloseUIModelFactory.create(
            position = mockPerpetualPositionData(),
            takeProfit = invalid,
            stopLoss = mockAutocloseField(TpslType.StopLoss),
            state = mockAutocloseViewState(),
        )
        assertNotNull(model.takeProfit.pnl)
        assertNull(model.stopLoss.pnl)
    }

    @Test
    fun errorSuppressedUntilShowErrorsSet() {
        val invalidTakeProfit = mockAutocloseField(TpslType.TakeProfit, price = 50.0, validation = AutocloseValidation.TRIGGER_MUST_BE_HIGHER)
        val hidden = AutocloseUIModelFactory.create(
            position = mockPerpetualPositionData(),
            takeProfit = invalidTakeProfit,
            stopLoss = mockAutocloseField(TpslType.StopLoss),
            state = mockAutocloseViewState(showsErrors = false),
        )
        val shown = AutocloseUIModelFactory.create(
            position = mockPerpetualPositionData(),
            takeProfit = invalidTakeProfit,
            stopLoss = mockAutocloseField(TpslType.StopLoss),
            state = mockAutocloseViewState(showsErrors = true),
        )
        assertFalse(hidden.takeProfit.showError)
        assertEquals(AutocloseValidation.TRIGGER_MUST_BE_HIGHER, shown.takeProfit.validation)
    }

    private fun model(leverage: UByte) = AutocloseUIModelFactory.create(
        position = mockPerpetualPositionData(position = mockPerpetualPosition(leverage = leverage)),
        takeProfit = mockAutocloseField(TpslType.TakeProfit),
        stopLoss = mockAutocloseField(TpslType.StopLoss),
        state = mockAutocloseViewState(),
    )
}
