package com.gemwallet.android.features.asset_select.presents.views

import com.gemwallet.android.ui.components.list_item.AssetItemUIModel

fun getAssetBadge(item: AssetItemUIModel): String {
    return if (item.asset.symbol == item.asset.name) "" else item.asset.symbol
}
