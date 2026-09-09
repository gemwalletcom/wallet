package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ui.components.list_item.property.SocialLinkUIModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency

class AssetMarketUIModel(
    val chain: Chain,
    val currency: Currency,
    val marketRows: List<MarketRowUIModel>,
    val contractRows: List<MarketRowUIModel>,
    val supplyRows: List<MarketRowUIModel>,
    val allTimeRows: List<MarketRowUIModel>,
    val links: List<SocialLinkUIModel>,
)
