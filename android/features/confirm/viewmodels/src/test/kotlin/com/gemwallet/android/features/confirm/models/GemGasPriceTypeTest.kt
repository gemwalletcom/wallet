package com.gemwallet.android.features.confirm.models

import com.wallet.core.primitives.FeeUnitType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
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

    @Test
    fun gasPriceDecimalsMapsByUnitType() {
        assertEquals(0, FeeUnitType.SatVb.gasPriceDecimals)
        assertEquals(9, FeeUnitType.Gwei.gasPriceDecimals)
        assertNull(FeeUnitType.Native.gasPriceDecimals)
    }

    @Test
    fun gasPriceSymbolMapsByUnitType() {
        assertEquals(FeeUnitType.SatVb.string, FeeUnitType.SatVb.gasPriceSymbol)
        assertEquals(FeeUnitType.Gwei.string, FeeUnitType.Gwei.gasPriceSymbol)
        assertNull(FeeUnitType.Native.gasPriceSymbol)
    }
}
