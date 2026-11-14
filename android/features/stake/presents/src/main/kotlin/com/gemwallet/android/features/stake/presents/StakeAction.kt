package com.gemwallet.android.features.stake.presents

import com.wallet.core.primitives.Delegation
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemStakeDestination

internal sealed interface StakeAction {
    data object Refresh : StakeAction
    data class Open(val destination: GemStakeDestination) : StakeAction
    data class RegionUnavailable(val topic: GemInfoTopic) : StakeAction
    data class OpenDelegation(val delegation: Delegation) : StakeAction
    data object Cancel : StakeAction
}
