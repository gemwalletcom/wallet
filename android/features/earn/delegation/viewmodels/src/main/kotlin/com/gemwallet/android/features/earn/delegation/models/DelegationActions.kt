package com.gemwallet.android.features.earn.delegation.models

import uniffi.gemstone.GemDelegationAction

sealed interface DelegationActions {
    object WithdrawalAction : DelegationActions

    object StakeAction : DelegationActions

    object UnstakeAction : DelegationActions

    object RedelegateAction : DelegationActions
}

fun GemDelegationAction.toDelegationAction(): DelegationActions? = when (this) {
    GemDelegationAction.STAKE -> DelegationActions.StakeAction
    GemDelegationAction.UNSTAKE -> DelegationActions.UnstakeAction
    GemDelegationAction.REDELEGATE -> DelegationActions.RedelegateAction
    GemDelegationAction.WITHDRAW -> DelegationActions.WithdrawalAction
    GemDelegationAction.DEPOSIT -> null
}