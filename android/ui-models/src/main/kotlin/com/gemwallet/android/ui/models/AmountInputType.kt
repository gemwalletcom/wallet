package com.gemwallet.android.ui.models

import com.gemwallet.android.math.parseNumber
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueConverter

enum class AmountInputType {
    Crypto {
        override fun getAmount(value: String, decimals: Int, price: Double): Crypto =
            Crypto(value.parseNumber(), decimals)

        override fun getInput(amount: Crypto?, decimals: Int, price: Double): String
                = amount?.value(decimals)?.stripTrailingZeros()?.toPlainString() ?: ""
    },
    Fiat {
        override fun getAmount(value: String, decimals: Int, price: Double): Crypto =
            ValueConverter().convertToAmount(value, price, decimals)

        override fun getInput(amount: Crypto?, decimals: Int, price: Double): String =
            amount?.convert(decimals, price)
                ?.value(decimals)?.stripTrailingZeros()?.toPlainString()
                ?: ""
    };

    abstract fun getAmount(value: String, decimals: Int, price: Double = 0.0): Crypto

    abstract fun getInput(amount: Crypto?, decimals: Int, price: Double): String
}