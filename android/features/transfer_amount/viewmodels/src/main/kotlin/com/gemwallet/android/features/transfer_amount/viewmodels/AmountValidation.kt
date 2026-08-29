package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.math.parseInputNumber
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAmountException
import uniffi.gemstone.GemAmountService
import java.math.BigInteger

private val amountService = GemAmountService()

object AmountValidation {
    fun parseAmount(asset: Asset, amount: String): Crypto {
        if (amount.isEmpty()) {
            throw AmountError.Required
        }
        val number = try {
            amount.parseInputNumber()
        } catch (_: Throwable) {
            throw AmountError.IncorrectAmount
        }
        return Crypto(number, asset.decimals)
    }

    fun validate(asset: Asset, amount: Crypto, availableBalance: BigInteger, minimumValue: BigInteger) {
        try {
            amountService.validate(amount.atomicValue.toString(), availableBalance.toString(), minimumValue.toString())
        } catch (error: GemAmountException) {
            throw when (error) {
                is GemAmountException.InvalidValue -> AmountError.IncorrectAmount
                is GemAmountException.Zero -> AmountError.ZeroAmount
                is GemAmountException.BelowMinimum -> AmountError.MinimumValue(ValueFormatter(style = ValueFormatter.Style.Full).string(minimumValue, asset))
                is GemAmountException.InsufficientBalance -> AmountError.InsufficientBalance(asset.symbol)
            }
        }
    }
}
