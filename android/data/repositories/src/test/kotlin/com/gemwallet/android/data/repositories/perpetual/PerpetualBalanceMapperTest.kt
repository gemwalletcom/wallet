package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.WalletId
import org.junit.Assert.assertEquals
import org.junit.Test

class PerpetualBalanceMapperTest {

    @Test
    fun `toDbBalance stores atomic strings and keeps display amounts`() {
        val balance = PerpetualBalance(available = 9.09, reserved = 1.5, withdrawable = 7.0)

        val db = balance.toDbBalance(WalletId("wallet-1"), HypercoreUSDC, updatedAt = 123L)

        assertEquals("9090000", db.available)
        assertEquals(9.09, db.availableAmount, 0.0)
        assertEquals("1500000", db.reserved)
        assertEquals(1.5, db.reservedAmount, 0.0)
        assertEquals("7000000", db.withdrawable)
        assertEquals(7.0, db.withdrawableAmount, 0.0)

        assertEquals(10.59, db.totalAmount, 1e-9)
        assertEquals(HypercoreUSDC.id.toIdentifier(), db.assetId)
        assertEquals("wallet-1", db.walletId)
        assertEquals(123L, db.updatedAt)
    }
}
