package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAssetCosmos
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import java.math.BigInteger
import uniffi.gemstone.GemAmountService

class AmountValidationTest {

    private val amountService = GemAmountService()
    private val asset = mockAssetCosmos()

    @Test
    fun `insufficient balance error uses asset symbol`() {
        val error = assertThrows(AmountError.InsufficientBalance::class.java) {
            AmountValidation.validate(amountService, asset, Crypto(BigInteger("200000000")), BigInteger("100000000"), BigInteger.ZERO)
        }
        assertEquals("ATOM", error.assetSymbol)
    }

    @Test
    fun `validate passes when amount equals balance`() {
        AmountValidation.validate(amountService, asset, Crypto(BigInteger("100000000")), BigInteger("100000000"), BigInteger.ZERO)
    }

    @Test
    fun `validate throws ZeroAmount for zero`() {
        assertThrows(AmountError.ZeroAmount::class.java) {
            AmountValidation.validate(amountService, asset, Crypto(BigInteger.ZERO), BigInteger("100000000"), BigInteger.ZERO)
        }
    }

    @Test
    fun `validate throws MinimumValue when below minimum`() {
        assertThrows(AmountError.MinimumValue::class.java) {
            AmountValidation.validate(amountService, asset, Crypto(BigInteger("500000")), BigInteger("100000000"), BigInteger("1000000"))
        }
    }

    @Test
    fun `parseAmount throws Required for empty input`() {
        assertThrows(AmountError.Required::class.java) {
            AmountValidation.parseAmount(asset, "")
        }
    }

    @Test
    fun `parseAmount throws IncorrectAmount for unparseable input`() {
        assertThrows(AmountError.IncorrectAmount::class.java) {
            AmountValidation.parseAmount(asset, "abc")
        }
    }
}
