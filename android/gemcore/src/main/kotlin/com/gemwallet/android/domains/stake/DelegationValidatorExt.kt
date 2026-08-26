package com.gemwallet.android.domains.stake

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import uniffi.gemstone.DelegationValidator as GemDelegationValidator

fun inactiveStakeValidator(chain: Chain, id: String, name: String): DelegationValidator {
    return DelegationValidator(
        chain = chain,
        id = id,
        name = name,
        isActive = false,
        commission = 0.0,
        apr = 0.0,
        providerType = StakeProviderType.Stake,
    )
}

fun DelegationValidator.toGem(): GemDelegationValidator = toJson()
