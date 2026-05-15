package com.gemwallet.android.domains.transaction

import com.wallet.core.primitives.TransactionDirection
import org.junit.Assert.assertEquals
import org.junit.Test

class AmountSignTest {

    @Test
    fun fromDirection_mapsEachDirection() {
        assertEquals(AmountSign.Incoming, AmountSign(TransactionDirection.Incoming))
        assertEquals(AmountSign.Outgoing, AmountSign(TransactionDirection.Outgoing))
        assertEquals(AmountSign.None, AmountSign(TransactionDirection.SelfTransfer))
    }

    @Test
    fun format_appliesSign() {
        assertEquals("+1.00 BTC", AmountSign.Incoming.format("1.00 BTC"))
        assertEquals("-1.00 BTC", AmountSign.Outgoing.format("1.00 BTC"))
        assertEquals("1.00 BTC", AmountSign.None.format("1.00 BTC"))
    }
}
