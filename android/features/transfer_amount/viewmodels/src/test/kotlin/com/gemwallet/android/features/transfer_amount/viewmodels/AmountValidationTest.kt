package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import java.math.BigDecimal
import java.math.BigInteger

class AmountValidationTest {

    @Test
    fun `insufficient balance error uses asset symbol`() {
        val error = assertThrows(AmountError.InsufficientBalance::class.java) {
            AmountValidation.validateBalance(
                assetInfo = mockAssetInfo(mockAssetCosmos()),
                amount = Crypto(BigInteger("200000000")),
                availableBalance = BigDecimal("100"),
            )
        }
        assertEquals("ATOM", error.assetSymbol)
    }

    @Test
    fun `validateBalance passes when amount equals balance`() {
        AmountValidation.validateBalance(
            assetInfo = mockAssetInfo(mockAssetCosmos()),
            amount = Crypto(BigInteger("100000000")),
            availableBalance = BigDecimal("100"),
        )
    }

    @Test
    fun `validateBalance throws ZeroAmount for zero`() {
        assertThrows(AmountError.ZeroAmount::class.java) {
            AmountValidation.validateBalance(
                assetInfo = mockAssetInfo(mockAssetCosmos()),
                amount = Crypto(BigInteger.ZERO),
                availableBalance = BigDecimal("100"),
            )
        }
    }

    @Test
    fun `validateAmount throws Required for empty input`() {
        assertThrows(AmountError.Required::class.java) {
            AmountValidation.validateAmount(mockAssetCosmos(), "", BigInteger.ZERO)
        }
    }

    @Test
    fun `validateAmount throws IncorrectAmount for unparseable input`() {
        assertThrows(AmountError.IncorrectAmount::class.java) {
            AmountValidation.validateAmount(mockAssetCosmos(), "abc", BigInteger.ZERO)
        }
    }

    @Test
    fun `validateAmount throws MinimumValue when below minimum`() {
        assertThrows(AmountError.MinimumValue::class.java) {
            AmountValidation.validateAmount(
                asset = mockAssetCosmos(),
                amount = "0.5",
                minValue = BigInteger("1000000"),
            )
        }
    }
}
