package com.gemwallet.android.ext

import com.wallet.core.primitives.PriceAlert
import uniffi.gemstone.PriceAlertFormatter

private val priceAlertFormatter = PriceAlertFormatter()

val PriceAlert.id: String
    get() = priceAlertFormatter.alertId(toGem())
