package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.PerpetualId

interface SetPerpetualPinned {
    suspend operator fun invoke(perpetualId: PerpetualId, pinned: Boolean)
}
