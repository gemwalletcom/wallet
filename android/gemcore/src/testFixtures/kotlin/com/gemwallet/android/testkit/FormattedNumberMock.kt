package com.gemwallet.android.testkit

import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberRounding
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemValueTone

fun mockFormattedNumber(
    value: Double = 1.0,
    unit: GemNumberUnit = GemNumberUnit.Currency(code = "USD"),
    tone: GemValueTone = GemValueTone.PLAIN,
    precision: GemPrecision = GemPrecision.Fraction(min = 2u, max = 2u),
    notation: GemNumberNotation = GemNumberNotation.PLAIN,
    exact: String? = null,
) = GemFormattedNumber(
    value = value,
    unit = unit,
    display = GemNumberDisplay.Number(precision = precision),
    notation = notation,
    tone = tone,
    rounding = GemNumberRounding.TO_NEAREST,
    exact = exact,
)
