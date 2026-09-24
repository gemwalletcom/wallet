package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.perpetual.cases.PerpetualSections
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualMarketSceneState
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
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
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetAction
import uniffi.gemstone.GemMarketsRefreshTrigger
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemPerpetualBalanceHeader
import uniffi.gemstone.GemPerpetualMarketCounts
import uniffi.gemstone.GemPerpetualMarketSection
import uniffi.gemstone.GemPerpetualMarketSession
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.perpetualBalanceHeader
import javax.inject.Inject

@HiltViewModel
class PerpetualMarketViewModel @Inject constructor(
    private val getPerpetuals: GetPerpetuals,
    private val getPositions: GetPerpetualPositions,
    private val getBalance: GetPerpetualBalance,
    private val getSession: GetSession,
    private val recentAssetsService: RecentAssetsService,
    private val service: GemPerpetualServiceInterface,
    private val perpetualObserver: PerpetualObserver,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
    private val connectionStatusObserver: ConnectionStatusObserver,
) : ViewModel() {

    private val session = MutableStateFlow(GemPerpetualMarketSession(query = "", isSearching = false))

    val isSearching: StateFlow<Boolean> = session.map { it.isSearching }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun setSearching(searching: Boolean) {
        session.update { it.onSearchingChanged(searching) }
    }

    val refreshIntervalMillis: StateFlow<Long> = connectionStatusObserver.refreshIntervalMillis(GemRefreshKind.MARKET)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private val query: StateFlow<String?> = session.map { it.searchQuery().takeIf(String::isNotEmpty) }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun setQuery(value: String) {
        session.update { it.onQueryChanged(value) }
    }
    val sceneState = MutableStateFlow<PerpetualMarketSceneState>(PerpetualMarketSceneState.Idle)
    private val perpetualSections = getPerpetuals.getPerpetualSections(query)
        .stateIn(viewModelScope, SharingStarted.Eagerly, PerpetualSections())
    val unpinnedPerpetuals = perpetualSections.map { it.markets }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val pinnedPerpetuals = perpetualSections.map { it.pinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val positions = combine(getPositions.getPerpetualPositions(), query) { items, q ->
        val needle = q.orEmpty()
        if (needle.isEmpty()) {
            items
        } else {
            items.filter {
                it.title.contains(needle, ignoreCase = true) ||
                    it.perpetualId.symbol.contains(needle, ignoreCase = true) ||
                    it.asset.symbol.contains(needle, ignoreCase = true) ||
                    it.asset.name.contains(needle, ignoreCase = true)
            }
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val positionRows: StateFlow<List<PerpetualPositionRowUIModel>> = positions
        .map { items -> items.map { PerpetualPositionRowUIModel(it.asset, it.listItem(context)) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val balanceHeader: StateFlow<GemPerpetualBalanceHeader?> = combine(
        getBalance.getBalance(),
        getSession().filterNotNull().map { it.wallet.type }.distinctUntilChanged(),
    ) { balance, walletType -> perpetualBalanceHeader(balance?.toGem(), walletType.toGem()) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val recent: StateFlow<List<Asset>> =
        recentAssetsService.getRecentAssets(RecentAssetsRequest(types = listOf(RecentActivityType.Perpetual)))
            .map { items -> items.map { it.asset } }
            .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val sections: StateFlow<List<GemPerpetualMarketSection>> = combine(positions, pinnedPerpetuals, unpinnedPerpetuals, recent, session) { positions, pinned, markets, recents, session ->
        session.sections(
            GemPerpetualMarketCounts(
                positions = positions.size.toUInt(),
                pinned = pinned.size.toUInt(),
                markets = markets.size.toUInt(),
                recents = recents.size.toUInt(),
            ),
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun onRefresh() {
        sceneState.update { PerpetualMarketSceneState.Refreshing }
        viewModelScope.launch(ioDispatcher) {
            refresh(GemMarketsRefreshTrigger.USER_REQUESTED)
            sceneState.update { PerpetualMarketSceneState.Idle }
        }
    }

    fun refreshMarkets() {
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
