package com.gemwallet.android.application.perpetual.cases

import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.PerpetualId
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemPerpetualPositionKind

interface BuildPerpetualParams {
    suspend fun position(perpetualId: PerpetualId, kind: GemPerpetualPositionKind): AmountParams.Perpetual?
    suspend fun close(perpetualId: PerpetualId): GemTransferData?
}
