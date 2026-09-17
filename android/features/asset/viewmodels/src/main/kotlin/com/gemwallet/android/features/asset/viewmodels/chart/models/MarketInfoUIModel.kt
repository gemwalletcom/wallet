package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.BlockExplorerLink

class MarketInfoUIModel(
    val model: ListItemModel,
    val layout: Layout = Layout.Plain,
    val explorerLink: BlockExplorerLink? = null,
) : MarketRowUIModel {
    enum class Layout { Plain, Badge, Address }
}

sealed class AllTimeUIModel(
    val date: Long,
    val value: Double,
    val percentage: Double,
    val model: ListItemModel,
) : MarketRowUIModel {
    class High(
        date: Long,
        value: Double,
        percentage: Double,
        model: ListItemModel,
    ) : AllTimeUIModel(date, value, percentage, model)

    class Low(
        date: Long,
        value: Double,
        percentage: Double,
        model: ListItemModel,
    ) : AllTimeUIModel(date, value, percentage, model)
}
