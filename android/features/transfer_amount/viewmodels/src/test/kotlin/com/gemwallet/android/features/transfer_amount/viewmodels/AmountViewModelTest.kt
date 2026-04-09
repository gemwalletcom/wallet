package com.gemwallet.android.features.transfer_amount.viewmodels

import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class AmountViewModelTest {

    @Test
    fun maxAmountAfterReserveReturnsZeroWhenReserveExceedsBalance() {
        val balance = BigInteger.valueOf(100)
        val reserve = BigInteger.valueOf(500)

        val result = AmountViewModel.maxAmountAfterReserve(balance, reserve)

        assertEquals(BigInteger.ZERO, result)
    }

    @Test
    fun maxAmountAfterReserveReturnsRemainderWhenBalanceSufficient() {
        val balance = BigInteger.valueOf(1000)
        val reserve = BigInteger.valueOf(300)

        val result = AmountViewModel.maxAmountAfterReserve(balance, reserve)

        assertEquals(BigInteger.valueOf(700), result)
    }

    @Test
    fun maxAmountAfterReserveReturnsZeroWhenEqual() {
        val balance = BigInteger.valueOf(500)
        val reserve = BigInteger.valueOf(500)

        val result = AmountViewModel.maxAmountAfterReserve(balance, reserve)

        assertEquals(BigInteger.ZERO, result)
    }
}
