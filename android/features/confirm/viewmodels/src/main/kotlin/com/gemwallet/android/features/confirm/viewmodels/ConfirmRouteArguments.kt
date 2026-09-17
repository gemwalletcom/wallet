package com.gemwallet.android.features.confirm.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.serializer.unpackRoutePayload
import com.gemwallet.android.ui.models.navigation.RouteArgument
import uniffi.gemstone.PaymentLink

internal fun SavedStateHandle.requirePaymentLink(): PaymentLink =
    checkNotNull(get<String>(RouteArgument.PaymentLink.key)?.let { unpackRoutePayload<PaymentLink>(it) }) {
        "Missing route argument: ${RouteArgument.PaymentLink.key}"
    }

internal fun SavedStateHandle.requireUrl(): String =
    checkNotNull(get<String>(RouteArgument.Url.key)) {
        "Missing route argument: ${RouteArgument.Url.key}"
    }
