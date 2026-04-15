package com.gemwallet.android.ui.components

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.AssetTag

@StringRes
fun AssetTag?.labelRes(): Int = when (this) {
    AssetTag.Trending,
    AssetTag.TrendingFiatPurchase -> R.string.assets_tags_trending
    AssetTag.Gainers -> R.string.assets_tags_gainers
    AssetTag.Losers -> R.string.assets_tags_losers
    AssetTag.New -> R.string.assets_tags_new
    AssetTag.Stablecoins -> R.string.assets_tags_stablecoins
    null -> R.string.common_all
}
