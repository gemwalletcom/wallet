package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ui.components.list_item.property.SocialLinkUIModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency

class AssetMarketUIModel(
    val chain: Chain,
    val currency: Currency,
    val sections: List<ChartSectionUIModel>,
)

sealed interface ChartSectionUIModel {
    data class PriceAlerts(val count: Int) : ChartSectionUIModel
    data object SetPriceAlert : ChartSectionUIModel
    data class Market(val rows: List<MarketRowUIModel>) : ChartSectionUIModel
    data class Links(val links: List<SocialLinkUIModel>) : ChartSectionUIModel
}
