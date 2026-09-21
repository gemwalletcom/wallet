package com.gemwallet.android.ui.models.navigation

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toNftAssetId
import com.gemwallet.android.serializer.unpackRoutePayload
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetId
import uniffi.gemstone.GemPaymentRecipient

fun SavedStateHandle.requireAssetId(argument: RouteArgument = RouteArgument.AssetId): AssetId {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    return checkNotNull(value.toAssetId()) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.requireChain(argument: RouteArgument = RouteArgument.Chain): Chain {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    return value.requireChain()
}

fun SavedStateHandle.requireNftAssetId(argument: RouteArgument = RouteArgument.NftAssetId): NFTAssetId {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    return checkNotNull(value.toNftAssetId()) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.optionalNft(argument: RouteArgument = RouteArgument.Nft): NFTAsset? {
    val value = get<String>(argument.key) ?: return null
    return checkNotNull(unpackRoutePayload<NFTAsset>(value)) { "Invalid route argument ${argument.key}: $value" }
}

fun SavedStateHandle.optionalPaymentRecipient(argument: RouteArgument = RouteArgument.Payment): GemPaymentRecipient? {
    val value = get<String>(argument.key) ?: return null
    return checkNotNull(unpackRoutePayload<GemPaymentRecipient>(value)) { "Invalid route argument ${argument.key}: $value" }
}
