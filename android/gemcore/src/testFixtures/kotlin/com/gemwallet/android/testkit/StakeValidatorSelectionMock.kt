package com.gemwallet.android.testkit

import uniffi.gemstone.GemStakeValidatorSelection
import uniffi.gemstone.GemValidatorRow

fun mockGemStakeValidatorSelection(
    validator: GemValidatorRow? = mockGemValidatorRow(),
    canSelect: Boolean = true,
) = GemStakeValidatorSelection(
    options = listOfNotNull(validator),
    recommended = emptyList(),
    validator = validator,
    canSelect = canSelect,
)
