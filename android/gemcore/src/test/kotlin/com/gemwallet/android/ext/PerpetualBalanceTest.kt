package com.gemwallet.android.ext

import com.wallet.core.primitives.PerpetualBalance
import org.junit.Assert.assertEquals
import org.junit.Test

class PerpetualBalanceTest {

    @Test
    fun `total sums available and reserved, ignoring withdrawable`() {
        val balance = PerpetualBalance(available = 50.0, reserved = 25.0, withdrawable = 40.0)

        assertEquals(75.0, balance.total, 0.0)
    }

    @Test
    fun `total with zero reserved equals available`() {
        val balance = PerpetualBalance(available = 100.0, reserved = 0.0, withdrawable = 0.0)

        assertEquals(100.0, balance.total, 0.0)
    }
}
