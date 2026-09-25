package com.gemwallet.android.testkit

import uniffi.gemstone.GemStakeValidatorOptions
import uniffi.gemstone.GemValidatorRow

fun mockGemStakeValidatorOptions(options: List<GemValidatorRow> = listOf(mockGemValidatorRow()), recommended: List<GemValidatorRow> = emptyList()) = GemStakeValidatorOptions(
    recommended = recommended,
    options = options,
)
