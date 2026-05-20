package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import uniffi.gemstone.Deeplink
import uniffi.gemstone.deeplinkDecodeUrl

internal fun String.toWebDeepLinkRoute(): NavKey? {
    return when (val deeplink = deeplinkDecodeUrl(this)) {
        is Deeplink.Asset -> deeplink.assetId.toAssetId()?.let { AssetRoute(it) }
        is Deeplink.Rewards -> ReferralRoute(code = deeplink.code?.takeIf(String::isNotBlank))
        else -> null
    }
}
