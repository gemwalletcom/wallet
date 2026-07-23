package com.gemwallet.android.ui.models

import com.gemwallet.android.math.parseInputNumber
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.Fiat
import java.math.BigInteger

enum class AmountInputType {
    Crypto {
        override fun getAmount(value: String, decimals: Int, price: Double): Crypto =
            Crypto(value.parseInputNumber(), decimals)
    },
    Fiat {
        override fun getAmount(value: String, decimals: Int, price: Double): Crypto =
            CryptoFiatConverter.toCryptoAtDisplayPrecision(Fiat(value.parseInputNumber()), decimals, price)
                ?: Crypto(BigInteger.ZERO)
    };

    abstract fun getAmount(value: String, decimals: Int, price: Double = 0.0): Crypto
}
