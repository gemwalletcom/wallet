package com.gemwallet.android.domains.confirm

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.feeRateDecimals
import com.gemwallet.android.ext.feeUnitType
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.FeeSelection
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.Config
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemFeeService
import java.math.BigInteger

class FeeDetailsModel(
    private val currentFee: FeeUIModel.FeeInfo,
    private val feeRates: List<GemFeeRate>,
    private val feeService: GemFeeService,
    private val maxMultiplier: Int,
    private val minimumCustomFeeRate: BigInteger?,
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
        maxMultiplier,
        minimumCustomFeeRate,
        feeService,
    )

    companion object {
        fun from(
            currentFee: FeeUIModel.FeeInfo,
            feeAssetInfo: AssetInfo,
            feeRates: List<GemFeeRate>,
            unitSymbol: String,
            feeService: GemFeeService,
        ): FeeDetailsModel {
            val chain = feeAssetInfo.asset.chain
            val feeUnitType = chain.feeUnitType()
            val feeConfig = Config().getFeeConfig(chain.string)
            val decimals = feeRateDecimals(feeUnitType, feeConfig, feeAssetInfo.asset.decimals)
            val selectedTotalFee = feeRates.firstOrNull { it.priority == currentFee.priority.toGem() }
                ?.let { feeService.totalFee(it.gasPriceType).toBigInteger() }
            return FeeDetailsModel(
                currentFee = currentFee,
                feeRates = feeRates,
                feeService = feeService,
                maxMultiplier = feeConfig.maxMultiplier.toInt(),
                minimumCustomFeeRate = feeConfig.minimumCustomFeeRate?.toLong()?.toBigInteger(),
                feeUnitType = feeUnitType,
                decimals = decimals,
                supportsCustomFee = feeConfig.customFeeEnabled && feeRates.size > 1,
                feeRateModels = feeRates.map { rate ->
                    FeeRateUIModel(
                        feeRate = rate,
                        feeAsset = feeAssetInfo,
                        feeUnitType = feeUnitType,
                        feeRateDecimals = decimals,
                        totalFee = feeService.totalFee(rate.gasPriceType).toBigInteger(),
                        selectedTotalFee = selectedTotalFee,
                        selectedFeeAmount = currentFee.amount,
                        unitSymbol = unitSymbol,
                    )
                },
            )
        }
    }
}
