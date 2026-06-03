package com.gemwallet.android.domains.perpetual.autoclose

import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class AutocloseValidatorTest {

    @Test
    fun nullAndZeroPrice() {
        val validator = AutocloseValidator(TpslType.TakeProfit, PerpetualDirection.Long, marketPrice = 100.0)
        assertNull(validator.error(price = null))
        assertEquals(AutocloseError.InvalidAmount, validator.error(price = 0.0))
        assertEquals(AutocloseError.InvalidAmount, validator.error(price = -1.0))
    }

    @Test
    fun longTakeProfitMustBeAboveMarket() {
        val validator = AutocloseValidator(TpslType.TakeProfit, PerpetualDirection.Long, marketPrice = 100.0)
        assertNull(validator.error(price = 110.0))
        assertEquals(AutocloseError.TriggerMustBeHigher, validator.error(price = 90.0))
    }

    @Test
    fun longStopLossMustBeBelowMarket() {
        val validator = AutocloseValidator(TpslType.StopLoss, PerpetualDirection.Long, marketPrice = 100.0)
        assertNull(validator.error(price = 90.0))
        assertEquals(AutocloseError.TriggerMustBeLower, validator.error(price = 110.0))
    }

    @Test
    fun shortTakeProfitMustBeBelowMarket() {
        val validator = AutocloseValidator(TpslType.TakeProfit, PerpetualDirection.Short, marketPrice = 100.0)
        assertNull(validator.error(price = 90.0))
        assertEquals(AutocloseError.TriggerMustBeLower, validator.error(price = 110.0))
    }

    @Test
    fun shortStopLossMustBeAboveMarket() {
        val validator = AutocloseValidator(TpslType.StopLoss, PerpetualDirection.Short, marketPrice = 100.0)
        assertNull(validator.error(price = 110.0))
        assertEquals(AutocloseError.TriggerMustBeHigher, validator.error(price = 90.0))
    }
}
