package com.gemwallet.android.ui.components.list_item.property

import android.content.Context
import androidx.annotation.DrawableRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemSocialLink
import uniffi.gemstone.LinkType

data class LinkRowUIModel(val url: String, val model: ListItemModel)

fun List<GemSocialLink>.linkRows(context: Context): List<LinkRowUIModel> = map { link ->
    LinkRowUIModel(
        url = link.url,
        model = ListItemModel(title = context.getString(link.linkType.stringRes()), subtitle = link.host, image = ListItemImage.Drawable(link.linkType.icon)),
    )
}

@get:DrawableRes
val LinkType.icon: Int
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
