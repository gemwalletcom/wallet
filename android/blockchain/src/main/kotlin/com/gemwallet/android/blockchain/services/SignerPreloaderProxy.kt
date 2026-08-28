package com.gemwallet.android.blockchain.services

import android.util.Log
import com.gemwallet.android.blockchain.gemstone.toFee
import com.gemwallet.android.domains.confirm.toConfirmInput
import com.gemwallet.android.ext.toFeePriority
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.SimulationResult
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmServiceInterface

class SignerPreloaderProxy(
    private val confirmService: GemConfirmServiceInterface,
) {
    private companion object {
        const val TAG = "SignerPreloader"
    }


    data class Preload(
        val signerParams: SignerParams,
        val simulation: SimulationResult?,
    )

    suspend fun preload(
        params: ConfirmParams,
        selection: FeeSelection,
        feeAssetSelection: FeeAssetSelection,
    ): Preload = withContext(Dispatchers.IO) {
        val result = confirmService.load(
            input = params.toConfirmInput(),
            options = GemConfirmLoadOptions(
                feeSelection = when (selection) {
                    is FeeSelection.Preset -> GemConfirmFeeSelection.Priority(selection.priority.string)
                    is FeeSelection.Custom -> GemConfirmFeeSelection.Custom(selection.gasPrice.toString())
                },
                feeAssetId = when (feeAssetSelection) {
                    FeeAssetSelection.Automatic -> null
                    is FeeAssetSelection.Selected -> feeAssetSelection.assetId.toIdentifier()
                },
            ),
        )
        val selectedPriority = result.selectedPriority.toFeePriority() ?: run {
            Log.e(TAG, "unsupported fee priority \"${result.selectedPriority}\"")
            FeePriority.Normal
        }
        val fee = result.fee.toFee(selectedPriority, AssetId(result.fee.feeAsset))
        val (rates, unsupported) = result.feeRates.partition { it.priority.toFeePriority() != null }
        if (unsupported.isNotEmpty()) {
            Log.e(TAG, "unsupported fee rates ${unsupported.joinToString { it.priority }}")
        }

        Preload(
            signerParams = SignerParams(
                input = params,
                confirmData = result,
                fee = fee,
                feeRates = rates,
            ),
            simulation = result.simulation?.decodeJson(),
        )
    }
}
