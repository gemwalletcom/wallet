package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReceiveRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SwapPairRoute
import com.wallet.core.primitives.FiatQuoteType
import uniffi.gemstone.Deeplink

internal fun Deeplink.toRoute(): NavKey? {
    return when (this) {
        is Deeplink.Asset -> assetId.toAssetId()?.let(::AssetRoute)
        is Deeplink.Rewards -> ReferralRoute(code = code?.takeIf(String::isNotBlank))
        is Deeplink.Receive -> assetId.toAssetId()?.let(::ReceiveRoute)
        is Deeplink.Buy -> assetId.toAssetId()?.let { FiatInputRoute(it, amount, FiatQuoteType.Buy) }
        is Deeplink.Sell -> assetId.toAssetId()?.let { FiatInputRoute(it, amount, FiatQuoteType.Sell) }
        is Deeplink.Swap -> assetId.toAssetId()?.let { SwapPairRoute(it, null) }
        Deeplink.Perpetuals -> PerpetualRoute
    }
}
