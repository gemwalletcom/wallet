package com.gemwallet.android.application.perpetual.coordinators

import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.PerpetualDirection

interface BuildPerpetualParams {
    suspend fun open(perpetualId: String, direction: PerpetualDirection): AmountParams.Perpetual?
    suspend fun increase(perpetualId: String): AmountParams.Perpetual?
    suspend fun reduce(perpetualId: String): AmountParams.Perpetual?
    suspend fun close(perpetualId: String): ConfirmParams.PerpetualParams?
}
