package com.gemwallet.android.domains.stake

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import uniffi.gemstone.GemDelegationValidator
import uniffi.gemstone.GemStakeProviderType

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

fun DelegationValidator.toGem(chain: uniffi.gemstone.Chain): GemDelegationValidator {
    return GemDelegationValidator(
        chain = chain,
        id = id,
        name = name,
        isActive = isActive,
        commission = commission,
        apr = apr,
        providerType = GemStakeProviderType.STAKE, // TODO: Fix on earn
    )
}
