package com.gemwallet.android.domains.confirm

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.feeRateDecimals
import com.gemwallet.android.ext.feeUnitType
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.FeeSelection
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.Config
import uniffi.gemstone.GemFeeRate

class FeeDetailsModel(
    private val currentFee: FeeUIModel.FeeInfo,
    private val feeRates: List<GemFeeRate>,
    val feeUnitType: FeeUnitType?,
    val decimals: Int,
    val supportsCustomFee: Boolean,
    val feeRateModels: List<FeeRateUIModel>,
) {
    fun customFee(input: String, selection: FeeSelection): CustomFee = CustomFee.from(
        input,
        currentFee,
        feeRates,
        selection,
        decimals,
    )

    companion object {
        fun from(
            currentFee: FeeUIModel.FeeInfo,
            feeAsset: FeeAssetUIModel,
            feeRates: List<GemFeeRate>,
            unitSymbol: String,
        ): FeeDetailsModel {
            val chain = feeAsset.asset.chain
            val feeUnitType = chain.feeUnitType()
            val feeConfig = Config().getFeeConfig(chain.string)
            val decimals = feeRateDecimals(feeUnitType, feeConfig, feeAsset.asset.decimals)
            val selectedTotalFee = feeRates.firstOrNull { it.priority == currentFee.priority.toGem() }
                ?.let { it.gasPriceType.totalFee() }
            return FeeDetailsModel(
                currentFee = currentFee,
                feeRates = feeRates,
                feeUnitType = feeUnitType,
                decimals = decimals,
                supportsCustomFee = feeConfig.customFeeEnabled && feeRates.size > 1,
                feeRateModels = feeRates.map { rate ->
                    FeeRateUIModel(
                        feeRate = rate,
                        feeAsset = feeAsset.priceValue,
                        feeUnitType = feeUnitType,
                        feeRateDecimals = decimals,
                        totalFee = rate.gasPriceType.totalFee(),
                        selectedTotalFee = selectedTotalFee,
                        selectedFeeAmount = currentFee.amount,
                        unitSymbol = unitSymbol,
                    )
                },
            )
        }
    }
}
