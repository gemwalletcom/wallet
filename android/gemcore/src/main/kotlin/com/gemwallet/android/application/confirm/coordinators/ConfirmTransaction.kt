package com.gemwallet.android.application.confirm.coordinators

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.SimulationResult
import kotlinx.coroutines.CoroutineScope

interface ConfirmTransaction {
    suspend operator fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
        simulation: SimulationResult? = null,
    ): String
}
