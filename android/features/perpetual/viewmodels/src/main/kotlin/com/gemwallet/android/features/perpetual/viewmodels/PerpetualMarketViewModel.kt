package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.ViewModel
import uniffi.gemstone.GemAssetAction
import androidx.lifecycle.viewModelScope
import android.util.Log
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.domains.perpetual.values.PerpetualBalance
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualMarketSceneState
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.RecentAssetsRequest
import com.wallet.core.primitives.RecentActivityType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import uniffi.gemstone.GemMarketsRefreshTrigger
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemRecentActivityService
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class PerpetualMarketViewModel @Inject constructor(
    private val getPerpetuals: GetPerpetuals,
    private val getPositions: GetPerpetualPositions,
    private val getBalance: GetPerpetualBalance,
    private val getRecentAssets: GetRecentAssets,
    private val service: GemPerpetualServiceInterface,
    private val recentActivity: GemRecentActivityService,
    private val perpetualObserver: PerpetualObserver,
) : ViewModel() {

    val query = MutableStateFlow<String?>(null)

    fun setQuery(value: String) {
        query.value = value.takeIf { it.isNotEmpty() }
    }
    val sceneState = MutableStateFlow<PerpetualMarketSceneState>(PerpetualMarketSceneState.Idle)
    private val perpetuals = getPerpetuals.getPerpetuals(query)
    val unpinnedPerpetuals = perpetuals.map { items -> items.filter { !it.isPinned } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val pinnedPerpetuals = perpetuals.map { items -> items.filter { it.isPinned } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val positions = combine(getPositions.getPerpetualPositions(), query) { items, q ->
        val needle = q?.trim().orEmpty()
        if (needle.isEmpty()) items else items.filter {
            it.name.contains(needle, ignoreCase = true) ||
                it.asset.symbol.contains(needle, ignoreCase = true) ||
                it.asset.name.contains(needle, ignoreCase = true)
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val balance = getBalance.getDisplayBalance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, EmptyPerpetualBalance)
    val recent: StateFlow<List<Asset>> =
        getRecentAssets(RecentAssetsRequest(types = listOf(RecentActivityType.Perpetual)))
            .map { items -> items.map { it.asset } }
            .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun onRefresh() {
        sceneState.update { PerpetualMarketSceneState.Refreshing }
        viewModelScope.launch(Dispatchers.IO) {
            syncMarkets(GemMarketsRefreshTrigger.USER_REQUESTED)
            syncPositions()
            delay(500)
            sceneState.update { PerpetualMarketSceneState.Idle }
        }
    }

    fun fetch() {
        viewModelScope.launch(Dispatchers.IO) { syncPositions() }
    }

    fun subscribeMarketPrices() {
        perpetualObserver.subscribe(GemPerpetualSubscription.MarketPrices)
    }

    fun unsubscribeMarketPrices() {
        perpetualObserver.unsubscribe(GemPerpetualSubscription.MarketPrices)
    }

    fun onTogglePin(perpetualId: PerpetualId) = viewModelScope.launch(Dispatchers.IO) {
        val item = (pinnedPerpetuals.value + unpinnedPerpetuals.value).firstOrNull { it.id == perpetualId } ?: return@launch
        runCatchingCancellable { service.setPinned(perpetualId.toIdentifier(), !item.isPinned) }
            .onFailure { Log.e(TAG, "pinning perpetual ${perpetualId.toIdentifier()} failed", it) }
    }

    fun onOpenPerpetual(asset: Asset) {
        viewModelScope.launch(Dispatchers.IO) {
            runCatchingCancellable { recentActivity.addRecent(GemAssetAction.OPEN, asset.toGem()) }
                .onFailure { Log.e(TAG, "recording recent perpetual ${asset.id.toIdentifier()} failed", it) }
        }
    }

    private suspend fun syncMarkets(trigger: GemMarketsRefreshTrigger) {
        runCatchingCancellable { service.syncMarketsIfNeeded(Chain.HyperCore.string, trigger) }
            .onFailure { Log.e(TAG, "perpetual markets sync failed", it) }
    }

    private suspend fun syncPositions() {
        runCatchingCancellable { service.syncCurrentPositions() }
            .onFailure { Log.e(TAG, "perpetual positions sync failed", it) }
    }

    private companion object {
        const val TAG = "PerpetualMarket"
    }
}

private object EmptyPerpetualBalance : PerpetualBalance {
    private val zero = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD).string(0.0)
    override val deposit: String = zero
    override val available: String = zero
    override val withdrawable: String = zero
    override val total: String = zero
}
