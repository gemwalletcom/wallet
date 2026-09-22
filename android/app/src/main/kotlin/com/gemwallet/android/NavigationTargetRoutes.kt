package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import uniffi.gemstone.GemNavigationTarget

internal fun GemNavigationTarget.routes(): List<NavKey> = when (this) {
    is GemNavigationTarget.Asset -> assetRoutes(asset.toPrimitives().id, isPerpetual)
    is GemNavigationTarget.Receive -> listOf(ReceiveRoute(asset.toPrimitives().id))
    is GemNavigationTarget.Fiat -> listOf(FiatInputRoute(asset.toPrimitives().id, amount, quoteType.toPrimitives()))
    is GemNavigationTarget.Swap -> listOf(SwapPairRoute(from.toPrimitives().id, to?.toPrimitives()?.id))
    GemNavigationTarget.Perpetuals -> listOf(PerpetualRoute)
    is GemNavigationTarget.Rewards -> listOf(ReferralRoute(code = code))
    GemNavigationTarget.Support -> listOf(SupportRoute)
    is GemNavigationTarget.Transaction -> assetRoutes(asset.toPrimitives().id, isPerpetual) + TransactionDetailsRoute(transaction.toPrimitives().id)
    GemNavigationTarget.None -> emptyList()
}

private fun assetRoutes(assetId: com.wallet.core.primitives.AssetId, isPerpetual: Boolean): List<NavKey> = when {
    isPerpetual -> listOf(PerpetualRoute, PerpetualPositionRoute(assetId))
    else -> listOf(AssetRoute(assetId))
}
