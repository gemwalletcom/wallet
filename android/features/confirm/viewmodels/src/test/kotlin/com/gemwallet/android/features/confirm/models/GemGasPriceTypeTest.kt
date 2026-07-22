package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.totalFee
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemGasPriceType
import java.math.BigInteger

class GemGasPriceTypeTest {

    @Test
    fun totalFeeMatchesGasPriceForEachVariant() {
        assertEquals(BigInteger("7"), GemGasPriceType.Regular(gasPrice = "7").totalFee())
        assertEquals(BigInteger("12"), GemGasPriceType.Eip1559(gasPrice = "5", priorityFee = "7").totalFee())
        assertEquals(
            BigInteger("9"),
            GemGasPriceType.Solana(gasPrice = "4", priorityFee = "5", unitPrice = "0").totalFee(),
        )
    }
}
