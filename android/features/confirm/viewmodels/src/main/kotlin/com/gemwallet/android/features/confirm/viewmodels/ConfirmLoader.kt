package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.SimulationResult
import uniffi.gemstone.GemTransferAmountResult
import javax.inject.Inject

class ConfirmLoader @Inject constructor(
    private val signerPreloader: SignerPreloaderProxy,
) {
    internal suspend fun load(
        walletId: String,
        params: ConfirmParams,
        feeSelection: FeeSelection,
        feeAssetSelection: FeeAssetSelection,
    ): ConfirmLoadResult {
        val preload = signerPreloader.preload(
            walletId = walletId,
            params = params,
            selection = feeSelection,
            feeAssetSelection = feeAssetSelection,
        )

        return ConfirmLoadResult(
            signerParams = preload.signerParams,
            simulation = preload.simulation,
            amount = preload.amount,
            feeAsset = preload.feeAsset,
        )
    }
}

internal data class ConfirmLoadResult(
    val signerParams: SignerParams,
    val simulation: SimulationResult?,
    val amount: GemTransferAmountResult,
    val feeAsset: Asset,
)
