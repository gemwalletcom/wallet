package com.gemwallet.android.features.transfer.viewmodels.amount

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.serializer.unpackRoutePayload
import com.gemwallet.android.ui.models.navigation.RouteArgument
import uniffi.gemstone.GemAmountRequest

internal fun SavedStateHandle.requireAmountRequest(): GemAmountRequest {
    val value = checkNotNull(get<String>(RouteArgument.Params.key)) {
        "Missing route argument: ${RouteArgument.Params.key}"
    }
    return checkNotNull(unpackRoutePayload<GemAmountRequest>(value)) {
        "Invalid route argument ${RouteArgument.Params.key}"
    }
}
