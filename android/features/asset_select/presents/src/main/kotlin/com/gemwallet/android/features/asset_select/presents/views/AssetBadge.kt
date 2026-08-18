package com.gemwallet.android.features.asset_select.presents.views

import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate

fun getAssetBadge(item: AssetInfoDataAggregate): String {
    return if (item.asset.symbol == item.asset.name) "" else item.asset.symbol
}
