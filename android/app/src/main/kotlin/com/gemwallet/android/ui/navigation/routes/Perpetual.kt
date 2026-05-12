package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.perpetual.views.market.PerpetualMarketNavScreen
import com.gemwallet.android.features.perpetual.views.position.PerpetualPositionNavScreen
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.Serializable

@Serializable
data object PerpetualRoute : NavKey

@Serializable
data class PerpetualPositionRoute(val assetId: AssetId) : NavKey

fun EntryProviderScope<NavKey>.perpetualScreen(
    onCancel: () -> Unit,
    onOpenPerpetualDetails: (AssetId) -> Unit,
    onTransaction: (TransactionId) -> Unit,
) {
    entry<PerpetualRoute> {
        PerpetualMarketNavScreen(
            onOpenPerpetualDetails = onOpenPerpetualDetails,
            onCancel = onCancel
        )
    }

    entry<PerpetualPositionRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        PerpetualPositionNavScreen(
            onClose = onCancel,
            onTransaction = onTransaction,
        )
    }
}
