package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import java.math.BigDecimal
import uniffi.gemstone.GemstoneException
import uniffi.gemstone.CryptoFiatConverter as GemCryptoFiatConverter

object CryptoFiatConverter {
    private val converter = GemCryptoFiatConverter()

    fun toFiat(crypto: Crypto, decimals: Int, price: Double): Fiat =
        Fiat(BigDecimal(converter.convertToFiat(crypto.atomicValue.toString(), decimals.toUInt(), price)))

    fun toFiatString(crypto: Crypto, decimals: Int, price: Double, currency: Currency): String =
        CurrencyFormatter(currency = currency).string(toFiat(crypto, decimals, price).atomicValue)

    fun toCrypto(fiat: Fiat, decimals: Int, price: Double): Crypto? =
        cryptoValue(fiat, decimals, price)?.let { Crypto(it, decimals) }

    fun toCryptoAtDisplayPrecision(fiat: Fiat, decimals: Int, price: Double): Crypto? =
        cryptoValue(fiat, decimals, price)?.let { Crypto(ValueFormatter(style = ValueFormatter.Style.Auto).rounded(it), decimals) }

    private fun cryptoValue(fiat: Fiat, decimals: Int, price: Double): BigDecimal? = try {
        BigDecimal(converter.convertToCrypto(fiat.atomicValue.toPlainString(), decimals.toUInt(), price))
    } catch (e: GemstoneException) {
        null
    }
}
