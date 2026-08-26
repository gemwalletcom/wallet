package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import uniffi.gemstone.GemExplorerService
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetAssetById
import com.gemwallet.android.application.assets.coordinators.GetAssetLinks
import com.gemwallet.android.application.assets.coordinators.GetAssetMarket
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.toModel
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class AssetChartViewModel internal constructor(
    getAssetById: GetAssetById,
    getAssetLinks: GetAssetLinks,
    getAssetMarket: GetAssetMarket,
    explorerService: GemExplorerService,
    getPriceAlerts: GetPriceAlerts,
    getCurrentCurrency: GetCurrentCurrency,
    val assetId: AssetId,
) : ViewModel() {

    private val explorerName = explorerService.getExplorerName(assetId.chain.string)

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
        getCurrentCurrency.getCurrency().distinctUntilChanged(),
    ) { asset, links, market, currency ->
        asset?.let {
            AssetMarketUIModel(
                asset = it,
                assetTitle = it.name,
                assetLinks = links.toModel(),
                currency = currency,
                marketInfo = market,
                explorerName = explorerName,
            )
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    @Inject
    constructor(
        getAssetById: GetAssetById,
        getAssetLinks: GetAssetLinks,
        getAssetMarket: GetAssetMarket,
        explorerService: GemExplorerService,
        getPriceAlerts: GetPriceAlerts,
        getCurrentCurrency: GetCurrentCurrency,
        savedStateHandle: SavedStateHandle,
    ) : this(
        getAssetById = getAssetById,
        getAssetLinks = getAssetLinks,
        getAssetMarket = getAssetMarket,
        explorerService = explorerService,
        getPriceAlerts = getPriceAlerts,
        getCurrentCurrency = getCurrentCurrency,
        assetId = savedStateHandle.requireAssetId(),
    )
}
