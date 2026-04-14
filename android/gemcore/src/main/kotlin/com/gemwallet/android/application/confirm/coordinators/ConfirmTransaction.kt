package com.gemwallet.android.application.confirm.coordinators

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import kotlinx.coroutines.CoroutineScope

interface ConfirmTransaction {
    data class Result(
        val txHash: String,
        val finishRoute: String,
    )

    suspend operator fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
    ): Result
}
