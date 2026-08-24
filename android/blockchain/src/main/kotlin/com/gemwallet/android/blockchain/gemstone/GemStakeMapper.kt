package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toChain
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import uniffi.gemstone.GemDelegationBase
import uniffi.gemstone.GemDelegationState
import uniffi.gemstone.GemDelegationValidator

internal fun GemDelegationValidator.toDTO(): DelegationValidator? {
    return DelegationValidator(
        chain = chain.toChain() ?: return null,
        id = id,
        name = name,
        isActive = isActive,
        commission = commission,
        apr = apr,
        providerType = StakeProviderType.Stake,
    )
}

internal fun GemDelegationBase.toDTO(): DelegationBase? {
    return DelegationBase(
        assetId = assetId.toAssetId() ?: return null,
        state = state.toDTO(),
        balance = balance,
        rewards = rewards,
        completionDate = completionDate,
        delegationId = delegationId,
        validatorId = validatorId,
        shares = shares,
    )
}

internal fun GemDelegationState.toDTO(): DelegationState = when (this) {
    GemDelegationState.ACTIVE -> DelegationState.Active
    GemDelegationState.PENDING -> DelegationState.Pending
    GemDelegationState.INACTIVE -> DelegationState.Inactive
    GemDelegationState.ACTIVATING -> DelegationState.Activating
    GemDelegationState.DEACTIVATING -> DelegationState.Deactivating
    GemDelegationState.AWAITING_WITHDRAWAL -> DelegationState.AwaitingWithdrawal
}
