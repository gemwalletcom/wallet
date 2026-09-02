package com.gemwallet.android.ext

import com.wallet.core.primitives.Currency

fun Currency.toGem(): uniffi.gemstone.Currency = string

fun uniffi.gemstone.Currency.toCurrency(): Currency =
    Currency.entries.firstOrNull { it.string == this }
        ?: throw IllegalStateException("Core returned a currency this build does not know: $this")
