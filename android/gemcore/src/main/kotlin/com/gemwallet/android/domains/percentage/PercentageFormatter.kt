package com.gemwallet.android.domains.percentage

import com.gemwallet.android.model.text
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.formattedPercentage
import java.util.Locale

fun Double?.formatAsPercentage(style: GemPercentageStyle = GemPercentageStyle.SIGNED, locale: Locale = Locale.getDefault()): String = this?.takeIf { it.isFinite() }?.let { formattedPercentage(it, style).text(locale) }.orEmpty()
