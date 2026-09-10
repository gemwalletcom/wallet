package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModelFactory
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModel
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemChartServiceInterface
import javax.inject.Inject

@HiltViewModel
class AssetChartViewModel internal constructor(
    getAssetTokenInfo: GetAssetTokenInfo,
    getAssetLinks: GetAssetLinks,
    getAssetMarket: GetAssetMarket,
    getWalletAssets: GetWalletAssets,
    private val chartService: GemChartServiceInterface,
    getPriceAlerts: GetPriceAlerts,
    getCurrentCurrency: GetCurrentCurrency,
    private val marketUIModelFactory: AssetMarketUIModelFactory,
    val assetId: AssetId,
) : ViewModel() {

    private val storedAssetInfo: AssetInfo? = getWalletAssets().value.firstOrNull { it.asset.id == assetId }

    private val assetInfo = getAssetTokenInfo(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedAssetInfo)

    private val links = getAssetLinks(assetId)
    private val market = getAssetMarket(assetId)
    private val priceAlerts = getPriceAlerts(assetId).map { alerts -> alerts.map { it.priceAlert } }

    val title = assetInfo
        .map { it?.asset?.name.orEmpty() }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedAssetInfo?.asset?.name.orEmpty())

    val marketUIModel = combine(assetInfo, links, market, priceAlerts, getCurrentCurrency.getCurrency(), ::marketUIModel)
        .flowOn(Dispatchers.IO)
        .stateIn(
            viewModelScope,
            SharingStarted.Eagerly,
            marketUIModel(storedAssetInfo, emptyList(), null, emptyList(), getCurrentCurrency.getCurrency().value),
        )

    private fun marketUIModel(
        assetInfo: AssetInfo?,
        links: List<AssetLink>,
        market: AssetMarket?,
        priceAlerts: List<PriceAlert>,
        currency: Currency,
    ): AssetMarketUIModel? = assetInfo?.let {
        marketUIModelFactory.create(
            asset = it.asset,
            currency = currency,
            sections = chartService.sections(
                asset = it.asset.toGem(),
                price = it.price?.price?.price,
                market = market?.toGem(),
                priceAlerts = priceAlerts.map { alert -> alert.toGem() },
                links = links.map { link -> link.toGem() },
            ),
        )
    }

    @Inject
    constructor(
        getAssetTokenInfo: GetAssetTokenInfo,
        getAssetLinks: GetAssetLinks,
        getAssetMarket: GetAssetMarket,
        getWalletAssets: GetWalletAssets,
        chartService: GemChartServiceInterface,
        getPriceAlerts: GetPriceAlerts,
        getCurrentCurrency: GetCurrentCurrency,
        marketUIModelFactory: AssetMarketUIModelFactory,
        savedStateHandle: SavedStateHandle,
    ) : this(
        getAssetTokenInfo = getAssetTokenInfo,
        getAssetLinks = getAssetLinks,
        getAssetMarket = getAssetMarket,
        getWalletAssets = getWalletAssets,
        chartService = chartService,
        getPriceAlerts = getPriceAlerts,
        getCurrentCurrency = getCurrentCurrency,
        marketUIModelFactory = marketUIModelFactory,
        assetId = savedStateHandle.requireAssetId(),
    )
}
