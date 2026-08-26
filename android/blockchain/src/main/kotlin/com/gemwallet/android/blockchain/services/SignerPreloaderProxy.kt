package com.gemwallet.android.blockchain.services

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
import com.wallet.core.primitives.SimulationResult
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmServiceInterface

class SignerPreloaderProxy(
    private val confirmService: GemConfirmServiceInterface,
) {

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
        val selectedPriority = requireNotNull(result.selectedPriority.toFeePriority())
        val fee = result.fee.toFee(selectedPriority, AssetId(result.fee.feeAsset))

        Preload(
            signerParams = SignerParams(
                input = params,
                selectedData = SignerParams.Data(metadata = result.metadata, fee = fee),
                feeRates = result.feeRates.filter { it.priority.toFeePriority() != null },
            ),
            simulation = result.simulation?.decodeJson(),
        )
    }
}
