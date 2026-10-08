package com.gemwallet.android.ui.navigation

import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId

internal fun assetIdArgument(assetId: AssetId): Pair<RouteArgument, String> = RouteArgument.AssetId to assetId.toIdentifier()

internal fun fiatAmountArgument(amount: Int?): Pair<RouteArgument, Int?> = RouteArgument.FiatAmount to amount

internal fun contactIdArgument(contactId: String): Pair<RouteArgument, String> = RouteArgument.ContactId to contactId

internal fun fromAssetIdArgument(assetId: AssetId?): Pair<RouteArgument, String?> = RouteArgument.FromAssetId to assetId?.toIdentifier()

internal fun toAssetIdArgument(assetId: AssetId?): Pair<RouteArgument, String?> = RouteArgument.ToAssetId to assetId?.toIdentifier()

internal fun paramsArgument(params: String): Pair<RouteArgument, String> = RouteArgument.Params to params

internal fun NavEntry<NavKey>.withOccurrenceContentKey(key: NavKey, occurrence: Int): NavEntry<NavKey> {
    val uniqueContentKey = if (occurrence == 0) contentKey else "$contentKey#$occurrence"
    val entry = this
    return NavEntry(
        key = key,
        contentKey = uniqueContentKey,
        metadata = metadata,
    ) {
        entry.Content()
    }
}
