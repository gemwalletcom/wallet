package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.numberFormat
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.GemCustomFeeEstimate
import uniffi.gemstone.GemCustomFeeInput
import uniffi.gemstone.GemFeeRateRows
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.customFeeEstimate

class FeeDetailsModel(private val currentFee: FeeUIModel.FeeInfo, private val feeAsset: FeeAssetUIModel, private val rows: GemFeeRateRows) {
    val feeUnitType: FeeUnitType = rows.unitType.toPrimitives()
    val decimals: Int = rows.unitDecimals.toInt()
    val supportsCustomFee: Boolean = rows.supportsCustomFee
    val showsOptions: Boolean = rows.showsOptions
    val customRate: GemLocalizedText? = rows.customRate

    fun feeRateModels(): List<FeeRateUIModel> = rows.rows.map { row -> FeeRateUIModel(row = row, feeAsset = feeAsset.priceValue) }

    val networkFeeAsset: Asset = currentFee.feeAsset

    fun customFee(input: String): GemCustomFeeEstimate = customFeeEstimate(
        GemCustomFeeInput(
            feeAsset = currentFee.feeAsset.toGem(),
            input = input,
            format = numberFormat(),
            rows = rows,
            loadedFee = currentFee.amount,
            price = currentFee.price,
            currency = currentFee.currency.toGem(),
        ),
    )
}
