package com.gemwallet.android.features.stake.presents

import com.wallet.core.primitives.Delegation

internal sealed interface StakeSceneAction {
    data object Refresh : StakeSceneAction
    data object ClaimRewards : StakeSceneAction
    data class OpenDelegation(val delegation: Delegation) : StakeSceneAction
    data object Cancel : StakeSceneAction
}
