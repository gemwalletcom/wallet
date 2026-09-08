package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.testkit.mockAssetCosmos
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAmountException
import uniffi.gemstone.GemBalanceRequirement
import java.math.BigInteger

class AmountErrorMapperTest {

    private val asset = mockAssetCosmos().toGem()

    @Test
    fun `insufficient balance error uses asset symbol`() {
        val error = GemAmountException.InsufficientBalance(asset, GemBalanceRequirement(BigInteger("200000000"), BigInteger("100000000"), BigInteger("100000000"))).toAmountError()
        assertEquals("ATOM", (error as AmountError.InsufficientBalance).assetSymbol)
    }

    @Test
    fun `zero stays silent`() {
        assertTrue(GemAmountException.Zero().toAmountError() is AmountError.None)
    }

    @Test
    fun `below minimum formats the minimum in the asset`() {
        val error = GemAmountException.BelowMinimum(asset, BigInteger("1500000")).toAmountError()
        assertEquals("1.5 ATOM", (error as AmountError.MinimumValue).minimumValue)
    }

    @Test
    fun `unparsable text and a missing price are an incorrect amount`() {
        assertTrue(GemAmountException.InvalidNumber().toAmountError() is AmountError.IncorrectAmount)
        assertTrue(GemAmountException.PriceMissing().toAmountError() is AmountError.IncorrectAmount)
    }
}
