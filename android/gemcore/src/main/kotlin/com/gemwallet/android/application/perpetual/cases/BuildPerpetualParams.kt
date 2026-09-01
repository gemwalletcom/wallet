package com.gemwallet.android.application.perpetual.cases

import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualModifyPositionType
import uniffi.gemstone.GemConfirmInput

interface BuildPerpetualParams {
    suspend fun open(perpetualId: PerpetualId, direction: PerpetualDirection): AmountParams.Perpetual?
    suspend fun increase(perpetualId: PerpetualId): AmountParams.Perpetual?
    suspend fun reduce(perpetualId: PerpetualId): AmountParams.Perpetual?
    suspend fun close(perpetualId: PerpetualId): GemConfirmInput?
    suspend fun modify(
        perpetualId: PerpetualId,
        modifyTypes: List<PerpetualModifyPositionType>,
        takeProfitOrderId: ULong?,
        stopLossOrderId: ULong?,
    ): GemConfirmInput?
}
