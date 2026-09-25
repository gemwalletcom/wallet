package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.store.queries.PriceQuery
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceData
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemListSection
import javax.inject.Inject

@HiltViewModel
class AssetChartViewModel internal constructor(
    priceQuery: PriceQuery,
    getWalletAssets: GetWalletAssets,
    private val chartService: GemChartServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
    private val ioDispatcher: CoroutineDispatcher,
    val assetId: AssetId,
) : ViewModel() {

    private val storedAssetInfo: AssetInfo? = getWalletAssets().value.firstOrNull { it.asset.id == assetId }

    private val priceData = priceQuery(assetId)
        .shareIn(viewModelScope, SharingStarted.Eagerly, replay = 1)

    val title = priceData
        .map { it?.asset?.name.orEmpty() }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedAssetInfo?.asset?.name.orEmpty())

    val sections = combine(priceData, getCurrentCurrency.getCurrency()) { data, _ -> sections(data) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private suspend fun sections(data: PriceData?): List<GemListSection> = data?.let {
        runCatchingCancellable {
            chartService.sections(
                asset = it.asset.toGem(),
                price = it.price?.price,
                market = it.market?.toGem(),
                priceAlerts = it.priceAlerts.map { alert -> alert.toGem() },
                links = it.links.map { link -> link.toGem() },
            )
        }.getOrNull()
    }.orEmpty()

    @Inject
    constructor(
        priceQuery: PriceQuery,
        getWalletAssets: GetWalletAssets,
        chartService: GemChartServiceInterface,
        getCurrentCurrency: GetCurrentCurrency,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
        savedStateHandle: SavedStateHandle,
    ) : this(
        priceQuery = priceQuery,
        getWalletAssets = getWalletAssets,
        chartService = chartService,
        getCurrentCurrency = getCurrentCurrency,
        ioDispatcher = ioDispatcher,
        assetId = savedStateHandle.requireAssetId(),
    )
}
