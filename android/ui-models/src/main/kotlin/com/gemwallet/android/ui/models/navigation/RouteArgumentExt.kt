package com.gemwallet.android.ui.models.navigation

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toNftAssetId
import com.gemwallet.android.serializer.unpackRoutePayload
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.PaymentRequest

fun SavedStateHandle.requireAssetId(argument: RouteArgument = RouteArgument.AssetId): AssetId {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    return checkNotNull(value.toAssetId()) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.optionalAssetId(argument: RouteArgument): AssetId? {
    val value = get<String>(argument.key) ?: return null
    return checkNotNull(value.toAssetId()) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.requireNftAssetId(argument: RouteArgument = RouteArgument.NftAssetId): NFTAssetId {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    return checkNotNull(value.toNftAssetId()) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.optionalNftAssetId(argument: RouteArgument = RouteArgument.NftAssetId): NFTAssetId? {
    val value = get<String>(argument.key) ?: return null
    return checkNotNull(value.toNftAssetId()) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.optionalPaymentRequest(argument: RouteArgument = RouteArgument.Payment): PaymentRequest? {
    val value = get<String>(argument.key) ?: return null
    return checkNotNull(unpackRoutePayload<PaymentRequest>(value)) { "Invalid route argument ${argument.key}: $value" }
}
