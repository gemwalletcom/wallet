package com.gemwallet.android.features.asset_select.presents.views

import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate

fun getAssetBadge(item: AssetInfoDataAggregate, showsSymbol: Boolean = true): String {
    if (!showsSymbol || item.asset.symbol == item.asset.name) {
        return ""
    }
    return item.asset.symbol
}
