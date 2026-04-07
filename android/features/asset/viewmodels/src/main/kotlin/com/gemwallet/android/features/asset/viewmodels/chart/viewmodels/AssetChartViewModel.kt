package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.asset.viewmodels.assetIdArg
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.toModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class AssetChartViewModel internal constructor(
    assetsRepository: AssetsRepository,
    getCurrentBlockExplorer: GetCurrentBlockExplorer,
    getPriceAlerts: GetPriceAlerts,
    sessionRepository: SessionRepository,
    val assetId: AssetId,
) : ViewModel() {

    private val explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(assetId.chain)

    val priceAlertsCount = getPriceAlerts(assetId)
        .map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    private val assetInfo = assetsRepository.getTokenInfo(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val links = assetsRepository.getAssetLinks(assetId)
    private val market = assetsRepository.getAssetMarket(assetId)

    val marketUIModel = combine(
        assetInfo,
        links,
        market,
        sessionRepository.getCurrency().distinctUntilChanged(),
    ) { assetInfo, links, market, currency ->
        assetInfo?.asset?.let { asset ->
            AssetMarketUIModel(
                asset = asset,
                assetTitle = asset.name,
                assetLinks = links.toModel(),
                currency = currency,
                marketInfo = market,
                explorerName = explorerName,
            )
        }
    }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    @Inject
    constructor(
        assetsRepository: AssetsRepository,
        getCurrentBlockExplorer: GetCurrentBlockExplorer,
        getPriceAlerts: GetPriceAlerts,
        sessionRepository: SessionRepository,
        savedStateHandle: SavedStateHandle,
    ) : this(
        assetsRepository = assetsRepository,
        getCurrentBlockExplorer = getCurrentBlockExplorer,
        getPriceAlerts = getPriceAlerts,
        sessionRepository = sessionRepository,
        assetId = checkNotNull(savedStateHandle.get<String>(assetIdArg)?.toAssetId()),
    )
}
