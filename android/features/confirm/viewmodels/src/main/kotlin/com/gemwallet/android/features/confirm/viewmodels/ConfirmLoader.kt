package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.blockchain.services.TransactionSimulationService
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.SimulationResult
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import javax.inject.Inject

class ConfirmLoader @Inject constructor(
    private val signerPreloader: SignerPreloaderProxy,
    private val transactionSimulationService: TransactionSimulationService,
) {
    internal suspend fun load(
        params: ConfirmParams,
        feeSelection: FeeSelection,
        feeAssetSelection: FeeAssetSelection,
    ): ConfirmLoadResult = coroutineScope {
        val preload = async {
            signerPreloader.preload(
                params = params,
                selection = feeSelection,
                feeAssetSelection = feeAssetSelection,
            )
        }
        val simulation = async { transactionSimulationService.simulate(params) }

        ConfirmLoadResult(
            signerParams = preload.await(),
            simulation = simulation.await(),
        )
    }
}

internal data class ConfirmLoadResult(
    val signerParams: SignerParams,
    val simulation: SimulationResult?,
)
