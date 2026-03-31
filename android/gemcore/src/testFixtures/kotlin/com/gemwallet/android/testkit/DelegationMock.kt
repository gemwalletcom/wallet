package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType

fun mockDelegationValidator(
    chain: Chain = Chain.Bitcoin,
    id: String = "validator-id",
    name: String = "Validator",
    isActive: Boolean = true,
    commission: Double = 5.0,
    apr: Double = 10.0,
    providerType: StakeProviderType = StakeProviderType.Stake,
) = DelegationValidator(
    chain = chain,
    id = id,
    name = name,
    isActive = isActive,
    commission = commission,
    apr = apr,
    providerType = providerType,
)

fun mockDelegation(
    assetId: AssetId = AssetId(Chain.Bitcoin),
    state: DelegationState = DelegationState.Active,
    balance: String = "0",
    shares: String = balance,
    rewards: String = "0",
    delegationId: String = "delegation-id",
    validatorId: String = "validator-id",
    validator: DelegationValidator = mockDelegationValidator(
        chain = assetId.chain,
        id = validatorId,
    ),
) = Delegation(
    base = DelegationBase(
        assetId = assetId,
        state = state,
        balance = balance,
        shares = shares,
        rewards = rewards,
        delegationId = delegationId,
        validatorId = validatorId,
    ),
    validator = validator,
)
