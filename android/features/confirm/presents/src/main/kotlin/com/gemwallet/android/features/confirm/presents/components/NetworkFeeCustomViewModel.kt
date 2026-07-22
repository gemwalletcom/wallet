package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.gemwallet.android.domains.confirm.CustomFee
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.math.NumberSanitizer
import com.gemwallet.android.model.FeeSelection
import uniffi.gemstone.GemFeeRate
import java.math.BigInteger

class NetworkFeeCustomViewModel(
    private val currentFee: FeeUIModel.FeeInfo,
    private val feeRates: List<GemFeeRate>,
    private val selection: FeeSelection,
    private val decimals: Int,
    private val maxMultiplier: Int,
    private val minimumCustomFeeRate: BigInteger?,
    initialRate: BigInteger?,
) {
    var input by mutableStateOf(initialRate?.let { CustomFee.format(it, decimals) } ?: "")
        private set

    private val custom by derivedStateOf {
        CustomFee.from(input, currentFee, feeRates, selection, decimals, maxMultiplier, minimumCustomFeeRate)
    }

    val placeholder: String get() = custom.placeholder
    val networkFee: FeeUIModel.FeeInfo get() = custom.networkFee
    val isOverMax: Boolean get() = custom.isOverMax
    val maxRateText: String get() = custom.maxRateText
    val isBelowMinimum: Boolean get() = custom.isBelowMinimum
    val minRateText: String get() = custom.minRateText
    val isConfirmEnabled: Boolean get() = custom.isConfirmEnabled
    val rate: BigInteger? get() = custom.rate

    fun onInputChange(value: String) {
        input = NumberSanitizer(maximumFractionDigits = decimals).sanitize(value)
    }
}
