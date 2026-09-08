package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetById
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModelFactory
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
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
    getAssetById: GetAssetById,
    getAssetLinks: GetAssetLinks,
    getAssetMarket: GetAssetMarket,
    private val chartService: GemChartServiceInterface,
    getPriceAlerts: GetPriceAlerts,
    getCurrentCurrency: GetCurrentCurrency,
    private val marketUIModelFactory: AssetMarketUIModelFactory,
    val assetId: AssetId,
) : ViewModel() {

    val priceAlertsCount = getPriceAlerts(assetId)
        .map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    private val asset = getAssetById(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val links = getAssetLinks(assetId)
    private val market = getAssetMarket(assetId)

    val title = asset
        .map { it?.name.orEmpty() }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val marketUIModel = combine(
        asset,
        links,
        market,
        getCurrentCurrency.getCurrency(),
    ) { asset, links, market, currency ->
        asset?.let {
            marketUIModelFactory.create(
                asset = it,
                currency = currency,
                rows = chartService.marketRows(it.toGem(), market?.toGem()),
                links = links,
            )
        }
    }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    @Inject
    constructor(
        getAssetById: GetAssetById,
        getAssetLinks: GetAssetLinks,
        getAssetMarket: GetAssetMarket,
        chartService: GemChartServiceInterface,
        getPriceAlerts: GetPriceAlerts,
        getCurrentCurrency: GetCurrentCurrency,
        marketUIModelFactory: AssetMarketUIModelFactory,
        savedStateHandle: SavedStateHandle,
    ) : this(
        getAssetById = getAssetById,
        getAssetLinks = getAssetLinks,
        getAssetMarket = getAssetMarket,
        chartService = chartService,
        getPriceAlerts = getPriceAlerts,
        getCurrentCurrency = getCurrentCurrency,
        marketUIModelFactory = marketUIModelFactory,
        assetId = savedStateHandle.requireAssetId(),
    )
}
