package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemSocialLink
import uniffi.gemstone.LinkType

class SocialLinkUIModel(
    val url: String,
    @get:StringRes val label: Int,
    @get:DrawableRes val icon: Int,
    val host: String? = null,
)

fun List<GemSocialLink>.toSocialLinks(): List<SocialLinkUIModel> = map { link ->
    SocialLinkUIModel(
        url = link.url,
        label = link.linkType.stringRes(),
        icon = link.linkType.icon,
        host = link.host,
    )
}

@get:DrawableRes
private val LinkType.icon: Int
    get() = when (this) {
        LinkType.X -> R.drawable.twitter
        LinkType.DISCORD -> R.drawable.discord
        LinkType.REDDIT -> R.drawable.reddit
        LinkType.TELEGRAM -> R.drawable.telegram
        LinkType.GIT_HUB -> R.drawable.github
        LinkType.YOU_TUBE -> R.drawable.youtube
        LinkType.FACEBOOK -> R.drawable.website
        LinkType.WEBSITE -> R.drawable.website
        LinkType.COINGECKO -> R.drawable.coingecko
        LinkType.OPEN_SEA -> R.drawable.opensea
        LinkType.INSTAGRAM -> R.drawable.instagram
        LinkType.MAGIC_EDEN -> R.drawable.magiceden
        LinkType.COIN_MARKET_CAP -> R.drawable.coinmarketcap
        LinkType.TIK_TOK -> R.drawable.tiktok
    }
