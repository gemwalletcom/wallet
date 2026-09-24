package com.gemwallet.android.features.stake.presents

import com.wallet.core.primitives.Delegation
import uniffi.gemstone.GemTransferData

internal sealed interface StakeSceneAction {
    data object Refresh : StakeSceneAction
    data class Confirm(val transfer: GemTransferData) : StakeSceneAction
    data class OpenDelegation(val delegation: Delegation) : StakeSceneAction
    data object Cancel : StakeSceneAction
}
