package com.gemwallet.android.ext

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.RedelegateData
import com.wallet.core.primitives.StakeType

fun uniffi.gemstone.DelegationBase.toPrimitives(): DelegationBase = DelegationBase(
    assetId = AssetId(assetId),
    state = state.toPrimitives(),
    balance = balance,
    shares = shares,
    rewards = rewards,
    completionDate = completionDate,
    delegationId = delegationId,
    validatorId = validatorId,
)

fun DelegationBase.toGem(): uniffi.gemstone.DelegationBase = uniffi.gemstone.DelegationBase(
    assetId = assetId.toIdentifier(),
    state = state.toGem(),
    balance = balance,
    shares = shares,
    rewards = rewards,
    completionDate = completionDate,
    delegationId = delegationId,
    validatorId = validatorId,
)

fun uniffi.gemstone.Delegation.toPrimitives(): Delegation = Delegation(base = base.toPrimitives(), validator = validator.toPrimitives())

fun Delegation.toGem(): uniffi.gemstone.Delegation = uniffi.gemstone.Delegation(base = base.toGem(), validator = validator.toGem())

fun uniffi.gemstone.RedelegateData.toPrimitives(): RedelegateData = RedelegateData(delegation = delegation.toPrimitives(), toValidator = toValidator.toPrimitives())

fun RedelegateData.toGem(): uniffi.gemstone.RedelegateData = uniffi.gemstone.RedelegateData(delegation = delegation.toGem(), toValidator = toValidator.toGem())

fun uniffi.gemstone.StakeType.toPrimitives(): StakeType = when (this) {
    is uniffi.gemstone.StakeType.Stake -> StakeType.Stake(v1.toPrimitives())
    is uniffi.gemstone.StakeType.Unstake -> StakeType.Unstake(v1.toPrimitives())
    is uniffi.gemstone.StakeType.Redelegate -> StakeType.Redelegate(v1.toPrimitives())
    is uniffi.gemstone.StakeType.Rewards -> StakeType.Rewards(v1.map { it.toPrimitives() })
    is uniffi.gemstone.StakeType.Withdraw -> StakeType.Withdraw(v1.toPrimitives())
    is uniffi.gemstone.StakeType.Freeze -> StakeType.Freeze(v1.toPrimitives())
    is uniffi.gemstone.StakeType.Unfreeze -> StakeType.Unfreeze(v1.toPrimitives())
}

fun StakeType.toGem(): uniffi.gemstone.StakeType = when (this) {
    is StakeType.Stake -> uniffi.gemstone.StakeType.Stake(content.toGem())
    is StakeType.Unstake -> uniffi.gemstone.StakeType.Unstake(content.toGem())
    is StakeType.Redelegate -> uniffi.gemstone.StakeType.Redelegate(content.toGem())
    is StakeType.Rewards -> uniffi.gemstone.StakeType.Rewards(content.map { it.toGem() })
    is StakeType.Withdraw -> uniffi.gemstone.StakeType.Withdraw(content.toGem())
    is StakeType.Freeze -> uniffi.gemstone.StakeType.Freeze(content.toGem())
    is StakeType.Unfreeze -> uniffi.gemstone.StakeType.Unfreeze(content.toGem())
}
