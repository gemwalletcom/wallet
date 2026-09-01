package com.gemwallet.android.blockchain.services

import android.util.Log
import com.gemwallet.android.blockchain.gemstone.toFee
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toIdentifier
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
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemTransferAmountResult
import com.wallet.core.primitives.Asset

class SignerPreloaderProxy(
    private val confirmService: GemConfirmServiceInterface,
) {
    private companion object {
        const val TAG = "SignerPreloader"
    }


    data class Preload(
        val signerParams: SignerParams,
        val simulation: SimulationResult?,
        val amount: GemTransferAmountResult,
        val feeAsset: Asset,
    )

    suspend fun preload(
        walletId: String,
        input: GemConfirmInput,
        selection: FeeSelection,
        feeAssetSelection: FeeAssetSelection,
    ): Preload = withContext(Dispatchers.IO) {
        val preload = confirmService.preload(
            walletId = walletId,
            input = input,
            options = GemConfirmLoadOptions(
                feeSelection = when (selection) {
                    is FeeSelection.Preset -> GemConfirmFeeSelection.Priority(selection.priority.toGem())
                    is FeeSelection.Custom -> GemConfirmFeeSelection.Custom(selection.gasPrice.toString())
                },
                feeAssetId = when (feeAssetSelection) {
                    FeeAssetSelection.Automatic -> null
                    is FeeAssetSelection.Selected -> feeAssetSelection.assetId.toIdentifier()
                },
            ),
        )
        val result = preload.confirmData
        val selectedPriority = result.selectedPriority.toPrimitives()
        val fee = result.fee.toFee(selectedPriority, AssetId(result.fee.feeAsset))
        val rates = result.feeRates

        Preload(
            signerParams = SignerParams(
                input = input,
                confirmData = result,
                fee = fee,
                feeRates = rates,
            ),
            simulation = result.simulation?.decodeJson(),
            amount = preload.amount,
            feeAsset = preload.feeAsset.toPrimitives(),
        )
    }
}
