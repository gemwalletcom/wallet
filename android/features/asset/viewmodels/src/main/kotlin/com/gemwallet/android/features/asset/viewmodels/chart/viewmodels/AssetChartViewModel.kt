package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetById
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModelFactory
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModel
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
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
    getWalletAssets: GetWalletAssets,
    private val chartService: GemChartServiceInterface,
    getPriceAlerts: GetPriceAlerts,
    getCurrentCurrency: GetCurrentCurrency,
    private val marketUIModelFactory: AssetMarketUIModelFactory,
    val assetId: AssetId,
) : ViewModel() {

    private val storedAsset: Asset? = getWalletAssets().value.firstOrNull { it.asset.id == assetId }?.asset

    val priceAlertsCount = getPriceAlerts(assetId)
        .map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    private val asset = getAssetById(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedAsset)

    private val links = getAssetLinks(assetId)
    private val market = getAssetMarket(assetId)

    val title = asset
        .map { it?.name.orEmpty() }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedAsset?.name.orEmpty())

    val marketUIModel = combine(asset, links, market, getCurrentCurrency.getCurrency(), ::marketUIModel)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, marketUIModel(storedAsset, emptyList(), null, getCurrentCurrency.getCurrency().value))

    private fun marketUIModel(asset: Asset?, links: List<AssetLink>, market: AssetMarket?, currency: Currency): AssetMarketUIModel? =
        asset?.let {
            marketUIModelFactory.create(
                asset = it,
                currency = currency,
                rows = chartService.marketRows(it.toGem(), market?.toGem()),
                links = links,
            )
        }

    @Inject
    constructor(
        getAssetById: GetAssetById,
        getAssetLinks: GetAssetLinks,
        getAssetMarket: GetAssetMarket,
        getWalletAssets: GetWalletAssets,
        chartService: GemChartServiceInterface,
        getPriceAlerts: GetPriceAlerts,
        getCurrentCurrency: GetCurrentCurrency,
        marketUIModelFactory: AssetMarketUIModelFactory,
        savedStateHandle: SavedStateHandle,
    ) : this(
        getAssetById = getAssetById,
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
