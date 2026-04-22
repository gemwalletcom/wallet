package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.math.parseNumber
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Asset
import java.math.BigDecimal
import java.math.BigInteger

object AmountValidation {

    fun validateAmount(asset: Asset, amount: String, minValue: BigInteger) {
        if (amount.isEmpty()) {
            throw AmountError.Required
        }
        try {
            amount.parseNumber()
        } catch (_: Throwable) {
            throw AmountError.IncorrectAmount
        }
        val crypto = Crypto(amount.parseNumber(), asset.decimals)
        if (minValue != BigInteger.ZERO && crypto.atomicValue < minValue) {
            throw AmountError.MinimumValue(asset.format(Crypto(minValue), decimalPlace = 2))
        }
    }

    fun validateBalance(
        assetInfo: AssetInfo,
        amount: Crypto,
        availableBalance: BigDecimal,
    ) {
        if (amount.atomicValue == BigInteger.ZERO) {
            throw AmountError.ZeroAmount
        }
        if (amount.value(assetInfo.asset.decimals) > availableBalance) {
            throw AmountError.InsufficientBalance(assetInfo.asset.symbol)
        }
    }
}
