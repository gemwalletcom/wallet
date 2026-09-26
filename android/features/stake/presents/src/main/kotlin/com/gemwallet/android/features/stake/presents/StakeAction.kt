package com.gemwallet.android.features.stake.presents

import com.wallet.core.primitives.Delegation
import uniffi.gemstone.GemTransferData

internal sealed interface StakeAction {
    data object Refresh : StakeAction
    data class Confirm(val transfer: GemTransferData) : StakeAction
    data class OpenDelegation(val delegation: Delegation) : StakeAction
    data object Cancel : StakeAction
}
