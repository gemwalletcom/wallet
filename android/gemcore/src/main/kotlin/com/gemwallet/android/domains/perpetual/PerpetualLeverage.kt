package com.gemwallet.android.domains.perpetual

import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.PerpetualProvider

fun Int.formatLeverage(): String = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.leverageText(toUByte()) }
