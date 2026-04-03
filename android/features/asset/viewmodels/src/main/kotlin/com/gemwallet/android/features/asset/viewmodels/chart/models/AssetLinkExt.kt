package com.gemwallet.android.features.asset.viewmodels.chart.models

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import com.gemwallet.android.ext.linkType
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.LinkType

fun List<AssetLink>.toModel() = mapNotNull(AssetLink::toModel)

private fun AssetLink.toModel(): AssetMarketUIModel.Link? {
    val linkType = linkType ?: return null
    return AssetMarketUIModel.Link(
        type = linkType.string,
        url = url,
        label = linkType.label,
        icon = linkType.icon,
    )
}

@get:StringRes
private val LinkType.label: Int
    get() = when (this) {
        LinkType.X -> R.string.social_x
        LinkType.Discord -> R.string.social_discord
        LinkType.Reddit -> R.string.social_reddit
        LinkType.Telegram -> R.string.social_telegram
        LinkType.GitHub -> R.string.social_github
        LinkType.YouTube -> R.string.social_youtube
        LinkType.Facebook -> R.string.social_facebook
        LinkType.Website -> R.string.social_website
        LinkType.Coingecko -> R.string.social_coingecko
        LinkType.OpenSea -> R.string.social_opensea
        LinkType.Instagram -> R.string.social_instagram
        LinkType.MagicEden -> R.string.social_magiceden
        LinkType.CoinMarketCap -> R.string.social_coinmarketcap
        LinkType.TikTok -> R.string.social_tiktok
    }

@get:DrawableRes
private val LinkType.icon: Int
    get() = when (this) {
        LinkType.X -> R.drawable.twitter
        LinkType.Discord -> R.drawable.discord
        LinkType.Reddit -> R.drawable.reddit
        LinkType.Telegram -> R.drawable.telegram
        LinkType.GitHub -> R.drawable.github
        LinkType.YouTube -> R.drawable.youtube
        LinkType.Facebook -> R.drawable.website
        LinkType.Website -> R.drawable.website
        LinkType.Coingecko -> R.drawable.coingecko
        LinkType.OpenSea -> R.drawable.opensea
        LinkType.Instagram -> R.drawable.instagram
        LinkType.MagicEden -> R.drawable.magiceden
        LinkType.CoinMarketCap -> R.drawable.coinmarketcap
        LinkType.TikTok -> R.drawable.tiktok
    }
