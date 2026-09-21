package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.GemFeeRateRows
import uniffi.gemstone.GemLocalizedText

class FeeDetailsModel(private val currentFee: FeeUIModel.FeeInfo, private val feeAsset: FeeAssetUIModel, private val rows: GemFeeRateRows) {
    val feeUnitType: FeeUnitType = rows.unitType.toPrimitives()
    val decimals: Int = rows.unitDecimals.toInt()
    val supportsCustomFee: Boolean = rows.supportsCustomFee
    val showsOptions: Boolean = rows.showsOptions
    val customRate: GemLocalizedText? = rows.customRate

    fun feeRateModels(): List<FeeRateUIModel> = rows.rows.map { row -> FeeRateUIModel(row = row, feeAsset = feeAsset.priceValue) }

    fun customFee(input: String): CustomFee = CustomFee.from(input, currentFee, rows)
}
