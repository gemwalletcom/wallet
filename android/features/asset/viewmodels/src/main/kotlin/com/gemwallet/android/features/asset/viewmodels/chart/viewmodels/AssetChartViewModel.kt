package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.asset.viewmodels.assetIdArg
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.toModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetChartViewModel @Inject constructor(
    private val assetsRepository: AssetsRepository,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
    private val getPriceAlerts: GetPriceAlerts,
    private val sessionRepository: SessionRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val assetIdStr = savedStateHandle.getStateFlow<String?>(assetIdArg, null)
    private val assetId = assetIdStr
        .mapNotNull { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val priceAlertsCount = assetId.filterNotNull()
        .flatMapLatest { getPriceAlerts(it) }
        .map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    private val assetInfo = assetId.filterNotNull()
        .flatMapLatest { assetsRepository.getTokenInfo(it).filterNotNull() }

    private val links = assetId.filterNotNull()
        .flatMapLatest { assetsRepository.getAssetLinks(it) }

    private val market = assetId.filterNotNull()
        .flatMapLatest { assetsRepository.getAssetMarket(it) }

    private val currency = sessionRepository.session()
        .map { it?.currency }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val marketUIModel = combine(assetInfo, links, market, currency.filterNotNull()) { assetInfo, links, market, currency ->
        val asset = assetInfo.asset
        AssetMarketUIModel(
            asset = asset,
            assetTitle = asset.name,
            assetLinks = links.toModel(),
            currency = currency,
            marketInfo = market,
            explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(asset.chain)
        )
    }
    .flowOn(Dispatchers.Default)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)
}
