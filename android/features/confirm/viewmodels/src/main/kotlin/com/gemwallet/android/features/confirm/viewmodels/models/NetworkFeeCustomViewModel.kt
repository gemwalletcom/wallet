package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.errorText
import uniffi.gemstone.GemCustomFeeCheck
import java.math.BigInteger

class NetworkFeeCustomViewModel(private val model: FeeDetailsModel, initialRate: BigInteger?) {
    private val decimals: Int = model.decimals

    var input by mutableStateOf(initialRate?.let { numberFormat().inputText(it.toString(), decimals.toUInt()) } ?: "")
        private set

    private val custom by derivedStateOf { model.customFee(input) }

    val placeholder: String get() = custom.placeholder?.text().orEmpty()
    val check: GemCustomFeeCheck get() = custom.check
    val isConfirmEnabled: Boolean get() = custom.isValid
    val rate: BigInteger? get() = custom.rate

    fun networkFeeItem(context: Context): ListItemModel = custom.fee.networkFeeListItem(context, model.networkFeeAsset)

    fun errorText(context: Context): String? = check.errorText(context)

    fun onInputChange(value: String) {
        input = numberFormat().sanitize(value, decimals.toUInt(), null)
    }
}
