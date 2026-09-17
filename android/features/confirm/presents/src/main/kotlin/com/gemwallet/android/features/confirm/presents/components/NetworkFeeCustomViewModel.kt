package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.gemwallet.android.domains.confirm.CustomFee
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import uniffi.gemstone.GemCustomFeeCheck
import uniffi.gemstone.GemNumberFormat
import java.text.DecimalFormatSymbols
import java.math.BigInteger

class NetworkFeeCustomViewModel(
    private val model: FeeDetailsModel,
    initialRate: BigInteger?,
) {
    private val decimals: Int = model.decimals

    var input by mutableStateOf(initialRate?.let { CustomFee.format(it, decimals) } ?: "")
        private set

    private val custom by derivedStateOf { model.customFee(input) }

    val placeholder: String get() = custom.placeholder
    val networkFee: FeeUIModel.FeeInfo get() = custom.networkFee
    val check: GemCustomFeeCheck get() = custom.check
    val maxRateText: String get() = custom.maxRateText
    val minRateText: String get() = custom.minRateText
    val isConfirmEnabled: Boolean get() = custom.isConfirmEnabled
    val rate: BigInteger? get() = custom.rate

    fun onInputChange(value: String) {
        input = GemNumberFormat(DecimalFormatSymbols.getInstance().decimalSeparator.toString()).sanitize(value, decimals.toUInt(), null)
    }
}
