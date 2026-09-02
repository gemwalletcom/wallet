package com.gemwallet.android.ui.models.navigation

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toNftAssetId
import com.gemwallet.android.model.PaymentRecipient
import com.gemwallet.android.serializer.unpackRoutePayload
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAssetId

fun SavedStateHandle.requireAssetId(argument: RouteArgument = RouteArgument.AssetId): AssetId {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
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

fun SavedStateHandle.optionalPaymentRecipient(argument: RouteArgument = RouteArgument.Payment): PaymentRecipient? {
    val value = get<String>(argument.key) ?: return null
    return checkNotNull(unpackRoutePayload<PaymentRecipient>(value)) { "Invalid route argument ${argument.key}: $value" }
}
