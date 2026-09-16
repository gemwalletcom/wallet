package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.GemValidatorRow

fun mockGemValidatorRow(
    validator: DelegationValidator = mockDelegationValidator(),
) = GemValidatorRow(
    validator = validator.toGem(),
    name = validator.name,
    imageUrl = "",
    placeholder = validator.name.take(1),
    provider = null,
)
