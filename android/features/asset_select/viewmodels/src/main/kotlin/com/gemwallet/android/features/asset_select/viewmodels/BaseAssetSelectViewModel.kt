package com.gemwallet.android.features.asset_select.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import android.util.Log
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.RecentActivityType
import uniffi.gemstone.GemAssetAction
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.AssetRowNaming
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ui.models.AssetToast
import com.gemwallet.android.ui.models.AssetToastEmitter
import com.gemwallet.android.ui.models.AssetToastEmitterImpl
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualId
import com.gemwallet.android.ext.toAssetId
import uniffi.gemstone.GemAssetConfigService
import uniffi.gemstone.GemAssetSelectionServiceInterface
import com.wallet.core.primitives.Wallet
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
open class BaseAssetSelectViewModel(
    getSession: GetSession,
    private val getRecentAssets: GetRecentAssets,
    protected val service: GemAssetSelectionServiceInterface,
    val search: SelectSearch,
    private val remoteSearch: Boolean = true,
) : ViewModel(), AssetToastEmitter by AssetToastEmitterImpl() {

    private val assetConfig = GemAssetConfigService()

    val queryState = TextFieldState()
    val chainFilter = MutableStateFlow<List<Chain>>(emptyList())
    val balanceFilter = MutableStateFlow(false)

    fun reset() {
        queryState.clearText()
        chainFilter.value = emptyList()
        balanceFilter.value = false
    }

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val isSearching = MutableStateFlow(false)

    val availableChains = session
        .map { session -> session?.wallet?.accounts?.map { it.chain } ?: emptyList() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    protected val currentQuery = snapshotFlow { queryState.text.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val searchRequests = currentQuery.debounce(SEARCH_DEBOUNCE_MS).distinctUntilChanged()

    private val filters = combine(
        session,
        currentQuery,
        chainFilter,
        balanceFilter,
    ) { session, query, chainFilter, hasBalance ->
        SelectAssetFilters(session = session, query = query, chainFilter = chainFilter, hasBalance = hasBalance, limit = assetsSearchLimit(query))
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetsContent = combine(
        filters,
        search.items(filters),
    ) { filters, items ->
        val chainFilter = filters?.chainFilter.orEmpty()
        val balanceFilter = filters?.hasBalance == true
        val wallet = session.value?.wallet
        items
            .filter { (chainFilter.isEmpty() || it.id().chain in chainFilter) && (!balanceFilter || it.balance.totalAmount > 0.0) }
            .map { item ->
                val owner = item.owner ?: wallet?.getAccount(item.asset.id.chain)
                val assetInfo = if (item.owner == owner) item else item.copy(owner = owner)
                assetInfo.toAssetInfoDataAggregate(AssetRowNaming.CanonicalNative)
            }
    }
    .flowOn(Dispatchers.IO)
    .shareIn(viewModelScope, SharingStarted.Eagerly, replay = 1)

    private val assets = assetsContent
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetInfoDataAggregate>())

    val popular = assets.map { items ->
        val popularIds = assetConfig.popularIds().mapNotNull { it.toAssetId() }
        items.filter { it.asset.id in popularIds }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetInfoDataAggregate>().toImmutableList())

    val pinned = assets.map { items ->
        items.filter { it.pinned }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetInfoDataAggregate>().toImmutableList())

    val unpinned = assets.map { items ->
        items.filter { !it.pinned }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetInfoDataAggregate>().toImmutableList())

    val recent = currentQuery
        .flatMapLatest { query ->
            if (query.isNotEmpty() || !showRecents) {
                flow { emit(emptyList()) }
            } else {
                getRecentAssets(RecentAssetsRequest(types = recentTypes, filters = assetFilters()))
            }
        }
    .map { items -> items.map { it.asset }.toImmutableList() }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<Asset>().toImmutableList())

    val uiState = combine(assetsContent, isSearching) { assets, isSearching ->
        when {
            assets.isNotEmpty() -> UIState.Idle
            isSearching -> UIState.Loading
            else -> UIState.Empty
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Idle)

    val isAddAssetAvailable = getSession().map { service.supportsTokens() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun onChangeVisibility(assetId: AssetId, visible: Boolean) = viewModelScope.launch {
        setVisibility(assetId, visible)
    }

    fun onAddToWallet(assetId: AssetId) = viewModelScope.launch {
        if (setVisibility(assetId, visible = true).isSuccess) {
            emitToast(AssetToast.AddedToWallet)
        }
    }

    fun onTogglePin(assetId: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        val item = assets.value.firstOrNull { it.asset.id == assetId }
        val willPin = item?.pinned != true
        runCatchingCancellable { service.setAssetPinned(assetId.toIdentifier(), willPin) }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
        item?.let { emitToast(AssetToast.Pin(it.asset.name, willPin)) }
    }

    private suspend fun setVisibility(assetId: AssetId, visible: Boolean): Result<Unit> = withContext(Dispatchers.IO) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), visible) }
            .onFailure { Log.e(TAG, "setting ${assetId.toIdentifier()} enabled=$visible failed", it) }
    }

    fun setChainFilter(chains: List<Chain>) {
        chainFilter.value = chains
    }

    fun onChainFilter(chain: Chain) {
        chainFilter.update {
            val chains = it.toMutableList()
            if (!chains.remove(chain)) {
                chains.add(chain)
            }
            chains.toList()
        }
    }

    fun onBalanceFilter(onlyWithBalance: Boolean) {
        balanceFilter.update { onlyWithBalance }
    }

    fun onClearFilters() {
        chainFilter.update { emptyList() }
        balanceFilter.update { false }
    }

    fun getAccount(assetId: AssetId): Account? {
        return session.value?.wallet?.getAccount(assetId)
    }

    init {
        if (remoteSearch) {
            viewModelScope.launch(Dispatchers.IO) {
                searchRequests.collectLatest { query ->
                    if (query.isEmpty()) return@collectLatest
                    isSearching.value = true
                    try {
                        runCatchingCancellable { searchRemote(query) }
                            .onFailure { Log.e(TAG, "search failed", it) }
                    } finally {
                        isSearching.value = false
                    }
                }
            }
        }
    }

    protected open suspend fun searchRemote(query: String) {
        service.searchAssets(query)
    }

    protected suspend fun setPerpetualPinned(perpetualId: PerpetualId, pinned: Boolean) {
        withContext(Dispatchers.IO) {
            runCatchingCancellable { service.setPerpetualPinned(perpetualId.toIdentifier(), pinned) }
                .onFailure { Log.e(TAG, "pinning perpetual ${perpetualId.toIdentifier()} failed", it) }
        }
    }

    fun updateRecent(asset: Asset, action: GemAssetAction) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { service.addRecent(action, asset.toGem()) }
            .onFailure { Log.e(TAG, "recording recent ${asset.id.toIdentifier()} failed", it) }
    }

    open val showRecents: Boolean get() = true

    open val action: GemAssetAction? get() = null

    val recentTypes: List<RecentActivityType>
        get() = action?.recentActivityTypes()?.map { it.toPrimitives() } ?: RecentActivityType.entries

    open fun assetFilters(): Set<AssetFilter> = emptySet()

    open fun assetsSearchLimit(query: String): Int = NO_QUERY_LIMIT

    private companion object {
        private const val TAG = "AssetSelect"
        private const val SEARCH_DEBOUNCE_MS = 250L
    }
}
