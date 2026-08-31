package com.gemwallet.android.domains.stake

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.DelegationValidator as GemDelegationValidator

fun DelegationValidator.toGem(): GemDelegationValidator = toJson()
