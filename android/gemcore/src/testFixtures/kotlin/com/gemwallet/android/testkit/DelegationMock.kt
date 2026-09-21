package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import uniffi.gemstone.GemClaimRewards
import uniffi.gemstone.GemClaimRewardsDestination
import java.math.BigInteger

fun mockDelegationValidator(chain: Chain = Chain.Bitcoin, id: String = "validator-id", apr: Double = 10.0, providerType: StakeProviderType = StakeProviderType.Stake) = DelegationValidator(
    chain = chain,
    id = id,
    name = "Validator",
    isActive = true,
    commission = 5.0,
    apr = apr,
    providerType = providerType,
)

fun mockDelegationBase(
    assetId: AssetId = mockAssetId(),
    state: DelegationState = DelegationState.Active,
    balance: BigInteger = BigInteger.ZERO,
    rewards: BigInteger = BigInteger.ZERO,
    delegationId: String = "delegation-id",
    validatorId: String = "validator-id",
) = DelegationBase(
    assetId = assetId,
    state = state,
    balance = balance,
    shares = balance,
    rewards = rewards,
    delegationId = delegationId,
    validatorId = validatorId,
)

fun mockDelegation(
    assetId: AssetId = mockAssetId(),
    balance: BigInteger = BigInteger.ZERO,
    rewards: BigInteger = BigInteger.ZERO,
    delegationId: String = "delegation-id",
    validatorId: String = "validator-id",
    validator: DelegationValidator = mockDelegationValidator(
        chain = assetId.chain,
        id = validatorId,
    ),
) = Delegation(
    base = mockDelegationBase(
        assetId = assetId,
        balance = balance,
        rewards = rewards,
        delegationId = delegationId,
        validatorId = validatorId,
    ),
    validator = validator,
)

fun mockClaimRewards() = GemClaimRewards(
    destination = GemClaimRewardsDestination.Amount(emptyList()),
)
