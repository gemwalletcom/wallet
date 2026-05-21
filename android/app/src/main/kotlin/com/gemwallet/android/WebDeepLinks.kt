package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import uniffi.gemstone.Deeplink

internal fun Deeplink.toRoute(): NavKey? {
    return when (this) {
        is Deeplink.Asset -> assetId.toAssetId()?.let { AssetRoute(it) }
        is Deeplink.Rewards -> ReferralRoute(code = code?.takeIf(String::isNotBlank))
        else -> null
    }
}
