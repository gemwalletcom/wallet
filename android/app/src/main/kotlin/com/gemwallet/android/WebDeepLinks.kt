package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset_select.presents.navigation.AssetsSearchRoute
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import java.net.URI

internal fun String.toWebDeepLinkRoute(): NavKey? {
    val uri = runCatching { URI(this) }.getOrNull() ?: return null
    if (!WebDeepLinkScheme.equals(uri.scheme, ignoreCase = true)) return null
    if (!WebDeepLinkHost.equals(uri.host, ignoreCase = true)) return null

    val segments = uri.pathSegments()
    return when (segments.firstOrNull()) {
        WebDeepLinkPathJoin -> ReferralRoute(
            code = segments.elementAtOrNull(1)
                ?: uri.rawQuery
                    .decodeUriQueryParameters()[WebDeepLinkCodeQuery]
                    ?.takeIf(String::isNotBlank),
        )
        WebDeepLinkPathTokens -> segments.toTokenRoute()
        else -> null
    }
}

private fun List<String>.toTokenRoute(): NavKey? {
    if (size == 1) return AssetsSearchRoute
    if (size !in 2..3) return null
    val chain = Chain.entries.firstOrNull { it.string == this[1] } ?: return null
    return AssetRoute(AssetId(chain = chain, tokenId = elementAtOrNull(2)))
}

private fun URI.pathSegments(): List<String> {
    return rawPath
        ?.split("/")
        ?.mapNotNull { it.takeIf(String::isNotBlank)?.decodeUriComponent() }
        .orEmpty()
}

private const val WebDeepLinkScheme = "https"
private const val WebDeepLinkHost = "gemwallet.com"
private const val WebDeepLinkPathJoin = "join"
private const val WebDeepLinkPathTokens = "tokens"
private const val WebDeepLinkCodeQuery = "code"
