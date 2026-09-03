package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.FeeSelection
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.GemFeeRateRows

class FeeDetailsModel(
    private val currentFee: FeeUIModel.FeeInfo,
    private val feeAsset: FeeAssetUIModel,
    private val rows: GemFeeRateRows,
) {
    val feeUnitType: FeeUnitType = rows.unitType.toPrimitives()
    val decimals: Int = rows.unitDecimals.toInt()
    val supportsCustomFee: Boolean = rows.supportsCustomFee

    fun feeRateModels(unitSymbol: String): List<FeeRateUIModel> = rows.rows.map { row ->
        FeeRateUIModel(
            row = row,
            feeAsset = feeAsset.priceValue,
            feeUnitType = feeUnitType,
            feeRateDecimals = decimals,
            unitSymbol = unitSymbol,
        )
    }

    fun customFee(input: String, selection: FeeSelection): CustomFee = CustomFee.from(input, currentFee, rows, decimals)
}
