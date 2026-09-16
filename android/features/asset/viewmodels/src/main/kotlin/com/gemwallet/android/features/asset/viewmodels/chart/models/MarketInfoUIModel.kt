package com.gemwallet.android.features.asset.viewmodels.chart.models

import androidx.annotation.StringRes
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.wallet.core.primitives.BlockExplorerLink

class MarketInfoUIModel(
    @param:StringRes val label: Int,
    val value: String,
    val layout: Layout = Layout.Plain,
    val badge: String? = null,
    val info: InfoSheetEntity? = null,
    val explorerLink: BlockExplorerLink? = null,
) : MarketRowUIModel {
    enum class Layout { Plain, Badge, Address }
}

sealed class AllTimeUIModel(
    val date: Long,
    val value: Double,
    val percentage: Double,
) : MarketRowUIModel {
    class High(
        date: Long,
        value: Double,
        percentage: Double,
    ) : AllTimeUIModel(date, value, percentage)

    class Low(
        date: Long,
        value: Double,
        percentage: Double,
    ) : AllTimeUIModel(date, value, percentage)
}