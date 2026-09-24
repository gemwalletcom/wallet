package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemValidatorRow

fun mockGemValidatorRow(validator: DelegationValidator = mockDelegationValidator(), apr: GemFormattedNumber? = null) = GemValidatorRow(
    validator = validator.toGem(),
    name = validator.name,
    imageUrl = "",
    placeholder = validator.name.take(1),
    provider = null,
    apr = GemLocalizedText.Apr(apr),
    explorer = null,
)
