package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.domains.connection.refreshInterval
import com.gemwallet.android.domains.perpetual.values.PerpetualBalance
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualMarketSceneState
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualMarketSectionUIModel
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.features.perpetual.viewmodels.models.uiModel
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.ui.components.perpetual.listItem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.RecentActivityType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetAction
import uniffi.gemstone.GemMarketsRefreshTrigger
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemPerpetualMarketCounts
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.PerpetualProvider

@HiltViewModel
class PerpetualMarketViewModel @Inject constructor(
    private val getPerpetuals: GetPerpetuals,
    private val getPositions: GetPerpetualPositions,
    private val getBalance: GetPerpetualBalance,
    private val recentAssetsService: RecentAssetsService,
    private val service: GemPerpetualServiceInterface,
    private val perpetualObserver: PerpetualObserver,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
    private val connectionStatusObserver: ConnectionStatusObserver,
) : ViewModel() {

    val isSearching = MutableStateFlow(false)

    val depositAssetId: AssetId = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.depositAsset() }.id.toAssetId()!!

    fun setSearching(searching: Boolean) {
        isSearching.value = searching
    }

    val refreshIntervalMillis: StateFlow<Long> = connectionStatusObserver.status
        .map { it.refreshInterval(GemRefreshKind.MARKET).toMillis() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)


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
            it.title.contains(needle, ignoreCase = true) ||
                it.perpetualId.symbol.contains(needle, ignoreCase = true) ||
                it.asset.symbol.contains(needle, ignoreCase = true) ||
                it.asset.name.contains(needle, ignoreCase = true)
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val positionRows: StateFlow<List<PerpetualPositionRowUIModel>> = positions
        .map { items -> items.map { PerpetualPositionRowUIModel(it.asset, it.listItem(context)) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val balance = getBalance.getDisplayBalance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, EmptyPerpetualBalance)
    val recent: StateFlow<List<Asset>> =
        recentAssetsService.getRecentAssets(RecentAssetsRequest(types = listOf(RecentActivityType.Perpetual)))
            .map { items -> items.map { it.asset } }
            .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val sections: StateFlow<List<PerpetualMarketSectionUIModel>> = combine(positions, pinnedPerpetuals, unpinnedPerpetuals, recent, query, isSearching) { values ->
        val positions = values[0] as List<*>
        val pinned = values[1] as List<*>
        val markets = values[2] as List<*>
        val recents = values[3] as List<*>
        val query = values[4] as String?
        val isSearching = values[5] as Boolean
        GemPerpetualMarketCounts(
            positions = positions.size.toUInt(),
            pinned = pinned.size.toUInt(),
            markets = markets.size.toUInt(),
            recents = recents.size.toUInt(),
        ).sections(isSearching, query.isNullOrEmpty()).list().map { it.uiModel(context) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun onRefresh() {
        sceneState.update { PerpetualMarketSceneState.Refreshing }
        viewModelScope.launch(ioDispatcher) {
            refresh(GemMarketsRefreshTrigger.USER_REQUESTED)
            delay(500)
            sceneState.update { PerpetualMarketSceneState.Idle }
        }
    }

    fun fetch() {
        viewModelScope.launch(ioDispatcher) { refresh(GemMarketsRefreshTrigger.SCHEDULED) }
    }

    fun subscribeMarketPrices() {
        perpetualObserver.subscribe(GemPerpetualSubscription.MarketPrices)
    }

    fun unsubscribeMarketPrices() {
        perpetualObserver.unsubscribe(GemPerpetualSubscription.MarketPrices)
    }

    fun onTogglePin(perpetualId: PerpetualId) = viewModelScope.launch(ioDispatcher) {
        val item = (pinnedPerpetuals.value + unpinnedPerpetuals.value).firstOrNull { it.id == perpetualId } ?: return@launch
        runCatchingCancellable { service.setPinned(perpetualId.toIdentifier(), !item.isPinned) }
            .onFailure { Log.e(TAG, "pinning perpetual ${perpetualId.toIdentifier()} failed", it) }
    }

    fun onOpenPerpetual(asset: Asset) {
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.addRecent(GemAssetAction.OPEN, asset.toGem()) }
                .onFailure { Log.e(TAG, "recording recent perpetual ${asset.id.toIdentifier()} failed", it) }
        }
    }

    private suspend fun refresh(trigger: GemMarketsRefreshTrigger) {
        runCatchingCancellable { service.refresh(trigger) }
            .onSuccess { failures -> failures.forEach { Log.e(TAG, "perpetual refresh failed at ${it.step}: ${it.message}") } }
            .onFailure { Log.e(TAG, "perpetual refresh failed", it) }
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
