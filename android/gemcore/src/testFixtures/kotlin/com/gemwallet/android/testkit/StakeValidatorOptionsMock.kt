package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemStakeValidatorOptions
import uniffi.gemstone.GemValidatorRow

fun mockGemStakeValidatorOptions(
    options: List<GemValidatorRow> = listOf(mockGemValidatorRow(validator = mockDelegationValidator().toGem(), name = mockDelegationValidator().name, placeholder = mockDelegationValidator().name.take(1), apr = GemLocalizedText.Apr(null))),
    recommended: List<GemValidatorRow> = emptyList(),
) = GemStakeValidatorOptions(
    recommended = recommended,
    options = options,
)
