package com.gemwallet.android.application.perpetual.coordinators

import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualModifyPositionType

interface BuildPerpetualParams {
    suspend fun open(perpetualId: PerpetualId, direction: PerpetualDirection): AmountParams.Perpetual?
    suspend fun increase(perpetualId: PerpetualId): AmountParams.Perpetual?
    suspend fun reduce(perpetualId: PerpetualId): AmountParams.Perpetual?
    suspend fun close(perpetualId: PerpetualId): ConfirmParams.PerpetualParams?
    suspend fun modify(
        perpetualId: PerpetualId,
        modifyTypes: List<PerpetualModifyPositionType>,
        takeProfitOrderId: ULong?,
        stopLossOrderId: ULong?,
    ): ConfirmParams.PerpetualParams?
}
