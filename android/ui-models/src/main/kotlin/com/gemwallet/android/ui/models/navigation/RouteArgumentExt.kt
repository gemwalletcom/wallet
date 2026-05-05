package com.gemwallet.android.ui.models.navigation

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.AssetId

fun SavedStateHandle.requireAssetId(argument: RouteArgument = RouteArgument.AssetId): AssetId {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    return checkNotNull(value.toAssetId()) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.optionalAssetId(argument: RouteArgument): AssetId? {
    val value = get<String>(argument.key) ?: return null
    return checkNotNull(value.toAssetId()) { "Invalid route argument ${argument.key}: $value" }
}
