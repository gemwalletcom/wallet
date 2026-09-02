package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetSmartChain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import java.math.BigInteger
import uniffi.gemstone.GemAmountStakeType
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemTransferBalance

class AmountValidationTest {

    private val asset = mockAssetCosmos()

    private fun balance(available: String) = GemTransferBalance(available, "0", "0", "0", 0u)

    @Test
    fun `insufficient balance error uses asset symbol`() {
        val error = assertThrows(AmountError.InsufficientBalance::class.java) {
            AmountValidation.validate(GemAmountType.Transfer, asset, Crypto(BigInteger("200000000")), balance("100000000"))
        }
        assertEquals("ATOM", error.assetSymbol)
    }

    @Test
    fun `validate passes when amount equals balance`() {
        AmountValidation.validate(GemAmountType.Transfer, asset, Crypto(BigInteger("100000000")), balance("100000000"))
    }

    @Test
    fun `validate throws ZeroAmount for zero`() {
        assertThrows(AmountError.ZeroAmount::class.java) {
            AmountValidation.validate(GemAmountType.Transfer, asset, Crypto(BigInteger.ZERO), balance("100000000"))
        }
    }

    @Test
    fun `validate throws MinimumValue when below minimum`() {
        val stake = GemAmountType.Stake(GemAmountStakeType.Stake)
        assertThrows(AmountError.MinimumValue::class.java) {
            AmountValidation.validate(stake, mockAssetSmartChain(), Crypto(BigInteger("500000000000000000")), balance("5000000000000000000"))
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
