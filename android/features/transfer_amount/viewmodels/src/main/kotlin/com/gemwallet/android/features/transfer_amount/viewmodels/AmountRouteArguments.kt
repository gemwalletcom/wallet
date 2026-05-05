package com.gemwallet.android.features.transfer_amount.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.models.navigation.RouteArgument

internal fun SavedStateHandle.requireAmountParams(): AmountParams {
    val value = checkNotNull(get<String>(RouteArgument.Params.key)) {
        "Missing route argument: ${RouteArgument.Params.key}"
    }
    return checkNotNull(AmountParams.unpack(value)) {
        "Invalid route argument ${RouteArgument.Params.key}"
    }
}
