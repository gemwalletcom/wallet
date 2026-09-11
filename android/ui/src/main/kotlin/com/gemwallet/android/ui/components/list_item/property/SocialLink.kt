package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
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
        label = link.linkType.label,
        icon = link.linkType.icon,
        host = link.host,
    )
}

@get:StringRes
private val LinkType.label: Int
    get() = when (this) {
        LinkType.X -> R.string.social_x
        LinkType.DISCORD -> R.string.social_discord
        LinkType.REDDIT -> R.string.social_reddit
        LinkType.TELEGRAM -> R.string.social_telegram
        LinkType.GIT_HUB -> R.string.social_github
        LinkType.YOU_TUBE -> R.string.social_youtube
        LinkType.FACEBOOK -> R.string.social_facebook
        LinkType.WEBSITE -> R.string.social_website
        LinkType.COINGECKO -> R.string.social_coingecko
        LinkType.OPEN_SEA -> R.string.social_opensea
        LinkType.INSTAGRAM -> R.string.social_instagram
        LinkType.MAGIC_EDEN -> R.string.social_magiceden
        LinkType.COIN_MARKET_CAP -> R.string.social_coinmarketcap
        LinkType.TIK_TOK -> R.string.social_tiktok
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
