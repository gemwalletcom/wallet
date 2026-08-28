package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.PerpetualId

interface SetPerpetualPinned {
    suspend operator fun invoke(perpetualId: PerpetualId, pinned: Boolean)
}
