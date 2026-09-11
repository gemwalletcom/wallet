package com.gemwallet.android.domains.stake

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.validatorDisplayName

fun DelegationValidator.displayName(): String = validatorDisplayName(toGem())
