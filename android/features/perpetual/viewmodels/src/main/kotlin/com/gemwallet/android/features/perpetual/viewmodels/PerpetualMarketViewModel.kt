package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalances
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetuals
import com.gemwallet.android.application.perpetual.coordinators.PerpetualObserver
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.application.perpetual.coordinators.TogglePerpetualPin
import com.gemwallet.android.domains.perpetual.values.PerpetualBalance
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualMarketSceneState
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import uniffi.gemstone.GemPerpetualSubscription
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class PerpetualMarketViewModel @Inject constructor(
    private val getPerpetuals: GetPerpetuals,
    private val getPositions: GetPerpetualPositions,
    private val getBalance: GetPerpetualBalances,
    private val syncPerpetuals: SyncPerpetuals,
    private val syncPerpetualPositions: SyncPerpetualPositions,
    private val togglePin: TogglePerpetualPin,
    private val getRecentAssets: GetRecentAssets,
    private val updateRecentAsset: UpdateRecentAsset,
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
    val balance = getBalance.getPerpetualBalance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, EmptyPerpetualBalance)
    val recent: StateFlow<List<Asset>> =
        getRecentAssets(RecentAssetsRequest(types = listOf(RecentType.Perpetual)))
            .map { items -> items.map { it.asset } }
            .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun onRefresh() {
        sceneState.update { PerpetualMarketSceneState.Refreshing }
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                syncPerpetuals.syncPerpetuals()
            }
            withContext(Dispatchers.IO) {
                syncPerpetualPositions.syncPerpetualPositions()
            }
            withContext(Dispatchers.IO) {
                delay(500)
                sceneState.update { PerpetualMarketSceneState.Idle }
            }
        }
    }

    fun fetch() {
        viewModelScope.launch(Dispatchers.IO) {
            syncPerpetualPositions.syncPerpetualPositions()
        }
    }

    fun subscribeMarketPrices() {
        perpetualObserver.subscribe(GemPerpetualSubscription.MarketPrices)
    }

    fun unsubscribeMarketPrices() {
        perpetualObserver.unsubscribe(GemPerpetualSubscription.MarketPrices)
    }

    fun onTogglePin(perpetualId: PerpetualId) {
        togglePin.togglePin(perpetualId)
    }

    fun onOpenPerpetual(assetId: AssetId) {
        viewModelScope.launch(Dispatchers.IO) {
            updateRecentAsset(assetId, RecentType.Perpetual)
        }
    }
}

private object EmptyPerpetualBalance : PerpetualBalance {
    private val zero = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD).string(0.0)
    override val deposit: String = zero
    override val available: String = zero
    override val withdrawable: String = zero
    override val total: String = zero
}
