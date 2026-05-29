package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ui.components.list_item.property.toSocialLinks
import com.wallet.core.primitives.AssetLink

fun List<AssetLink>.toModel() = toSocialLinks().map {
    AssetMarketUIModel.Link(
        type = it.type,
        url = it.url,
        label = it.label,
        icon = it.icon,
        host = it.host,
    )
}
