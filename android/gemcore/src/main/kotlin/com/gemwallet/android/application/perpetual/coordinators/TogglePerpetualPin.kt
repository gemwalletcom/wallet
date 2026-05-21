package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.PerpetualId

interface TogglePerpetualPin {
    fun togglePin(perpetualId: PerpetualId)
}
