package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.domains.fiat.FiatConfig
import io.mockk.every
import io.mockk.mockkObject
import io.mockk.unmockkObject
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class AmountValidatorTest {

    @Before
    fun setUp() {
        mockkObject(FiatConfig)
        every { FiatConfig.maximumAmount } returns 10000
    }

    @After
    fun tearDown() {
        unmockkObject(FiatConfig)
    }

    @Test
    fun `valid amount above minimum returns true`() {
        val validator = AmountValidator(20.0)
        assertTrue(validator.validate("50"))
        assertNull(validator.error)
    }

    @Test
    fun `amount equal to minimum returns true`() {
        val validator = AmountValidator(20.0)
        assertTrue(validator.validate("20"))
        assertNull(validator.error)
    }

    @Test
    fun `amount below minimum returns MinimumAmount error`() {
        val validator = AmountValidator(20.0)
        assertFalse(validator.validate("10"))
        assertEquals(BuyError.MinimumAmount, validator.error)
    }

    @Test
    fun `zero amount with positive minimum returns MinimumAmount error`() {
        val validator = AmountValidator(20.0)
        assertFalse(validator.validate("0"))
        assertEquals(BuyError.MinimumAmount, validator.error)
    }

    @Test
    fun `empty string with positive minimum returns MinimumAmount error`() {
        val validator = AmountValidator(20.0)
        assertFalse(validator.validate(""))
        assertEquals(BuyError.MinimumAmount, validator.error)
    }

    @Test
    fun `sell validator with zero minimum rejects zero`() {
        val validator = AmountValidator(0.0)
        assertFalse(validator.validate("0"))
        assertEquals(BuyError.EmptyAmount, validator.error)
    }

    @Test
    fun `sell validator with zero minimum accepts small amount`() {
        val validator = AmountValidator(0.0)
        assertTrue(validator.validate("1"))
        assertNull(validator.error)
    }

    @Test
    fun `error is cleared on successful validation`() {
        val validator = AmountValidator(20.0)
        assertFalse(validator.validate("5"))
        assertEquals(BuyError.MinimumAmount, validator.error)

        assertTrue(validator.validate("50"))
        assertNull(validator.error)
    }

    @Test
    fun `amount at maximum validates successfully`() {
        val validator = AmountValidator(20.0)
        assertTrue(validator.validate(FiatConfig.maximumAmount.toString()))
        assertNull(validator.error)
    }

    @Test
    fun `amount above maximum returns MaximumAmount error`() {
        val validator = AmountValidator(20.0)
        assertFalse(validator.validate((FiatConfig.maximumAmount + 1).toString()))
        assertEquals(BuyError.MaximumAmount, validator.error)
    }

    @Test
    fun `custom max amount is enforced`() {
        val validator = AmountValidator(5.0, maxValue = 500.0)
        assertFalse(validator.validate("501"))
        assertEquals(BuyError.MaximumAmount, validator.error)
    }

    @Test
    fun `decimal amount validates correctly`() {
        val validator = AmountValidator(20.0)
        assertTrue(validator.validate("25.50"))
        assertNull(validator.error)
    }
}
