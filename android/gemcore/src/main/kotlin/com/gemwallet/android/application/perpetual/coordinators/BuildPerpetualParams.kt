package com.gemwallet.android.application.perpetual.coordinators

import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId

interface BuildPerpetualParams {
    suspend fun open(perpetualId: PerpetualId, direction: PerpetualDirection): AmountParams.Perpetual?
    suspend fun increase(perpetualId: PerpetualId): AmountParams.Perpetual?
    suspend fun reduce(perpetualId: PerpetualId): AmountParams.Perpetual?
    suspend fun close(perpetualId: PerpetualId): ConfirmParams.PerpetualParams?
}
