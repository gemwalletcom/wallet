package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.perpetual.views.market.PerpetualMarketNavScreen
import com.gemwallet.android.features.perpetual.views.position.PerpetualPositionNavScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data object PerpetualRoute : NavKey

@Serializable
data class PerpetualPositionRoute(val perpetualId: String) : NavKey

fun EntryProviderScope<NavKey>.perpetualScreen(
    onCancel: () -> Unit,
    onOpenPerpetualDetails: (String) -> Unit,
) {
    entry<PerpetualRoute> {
        PerpetualMarketNavScreen(
            onOpenPerpetualDetails = onOpenPerpetualDetails,
            onCancel = onCancel
        )
    }

    entry<PerpetualPositionRoute>(
        metadata = { key -> routeArguments(RouteArgument.PerpetualId to key.perpetualId) },
    ) {
        PerpetualPositionNavScreen(onClose = onCancel)
    }
}
