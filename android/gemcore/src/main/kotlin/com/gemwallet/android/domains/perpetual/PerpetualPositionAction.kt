package com.gemwallet.android.domains.perpetual

import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemPerpetualTransferData

val GemPerpetualPositionAction.data: GemPerpetualTransferData
    get() = when (this) {
        is GemPerpetualPositionAction.Open -> data
        is GemPerpetualPositionAction.Increase -> data
        is GemPerpetualPositionAction.Reduce -> data
    }
