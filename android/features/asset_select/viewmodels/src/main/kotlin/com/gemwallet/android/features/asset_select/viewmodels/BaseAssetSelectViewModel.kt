package com.gemwallet.android.features.asset_select.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.asset_select.cases.SwitchAssetVisibility
import com.gemwallet.android.application.assets.cases.SetAssetPinned
import com.gemwallet.android.application.asset_select.cases.UpdateRecentAsset
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.ext.assetType
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.model.RecentType
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
import com.gemwallet.android.ext.toAssetId
import uniffi.gemstone.GemAssetConfigService
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
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

private val assetConfig = GemAssetConfigService()

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
open class BaseAssetSelectViewModel(
    getSession: GetSession,
    private val getRecentAssets: GetRecentAssets,
    private val updateRecentAsset: UpdateRecentAsset,
    private val switchAssetVisibility: SwitchAssetVisibility,
    private val setAssetPinned: SetAssetPinned,
    private val searchTokensCase: SearchTokens,
    val search: SelectSearch,
    private val remoteSearch: Boolean = true,
) : ViewModel(), AssetToastEmitter by AssetToastEmitterImpl() {

    val queryState = TextFieldState()
    val chainFilter = MutableStateFlow<List<Chain>>(emptyList())
    val balanceFilter = MutableStateFlow(false)

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val isSearching = MutableStateFlow(false)

    val availableChains = session
        .map { session -> session?.wallet?.accounts?.map { it.chain } ?: emptyList() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    protected val currentQuery = snapshotFlow { queryState.text.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val searchRequests = combine(currentQuery.debounce(SEARCH_DEBOUNCE_MS), session) { query, session ->
        SearchRequest(query, session?.currency ?: Currency.USD, walletSearchChains(session?.wallet))
    }.distinctUntilChanged()

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
        items.filter { it.pinned && it.balanceEnabled }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetInfoDataAggregate>().toImmutableList())

    val unpinned = assets.map { items ->
        items.filter { !it.pinned || !it.balanceEnabled }.toImmutableList()
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

    val isAddAssetAvailable = getSession().map { session ->
        session?.wallet?.accounts?.any { it.chain.assetType() != null } == true
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun onChangeVisibility(assetId: AssetId, visible: Boolean) = viewModelScope.launch {
        setVisibility(assetId, visible)
    }

    fun onAddToWallet(assetId: AssetId) = viewModelScope.launch {
        if (setVisibility(assetId, visible = true)) {
            emitToast(AssetToast.AddedToWallet)
        }
    }

    fun onTogglePin(assetId: AssetId) = viewModelScope.launch {
        val session = session.value ?: return@launch
        session.wallet.getAccount(assetId.chain) ?: return@launch
        val item = assets.value.firstOrNull { it.asset.id == assetId }
        val willPin = item?.pinned != true
        setAssetPinned(assetId, willPin)
        item?.let { emitToast(AssetToast.Pin(it.asset.name, willPin)) }
    }

    private suspend fun setVisibility(assetId: AssetId, visible: Boolean): Boolean {
        val session = session.value ?: return false
        session.wallet.getAccount(assetId.chain) ?: return false
        switchAssetVisibility(session.wallet.id, assetId, visible)
        return true
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
                searchRequests.collectLatest { (query, currency, chains) ->
                    isSearching.value = query.isNotEmpty()
                    try {
                        runCatchingCancellable {
                            searchTokensCase.search(query, currency, chains)
                        }
                    } finally {
                        isSearching.value = false
                    }
                }
            }
        }
    }

    protected fun walletSearchChains(wallet: Wallet?): List<Chain> = when (wallet?.type) {
        WalletType.Multicoin -> emptyList()
        WalletType.Single, WalletType.PrivateKey, WalletType.View -> listOfNotNull(wallet.accounts.firstOrNull()?.chain)
        null -> emptyList()
    }

    fun updateRecent(assetId: AssetId, type: RecentType) = viewModelScope.launch(Dispatchers.IO) {
        updateRecentAsset(assetId, type)
    }

    open val showRecents: Boolean get() = true

    open val recentTypes: List<RecentType> get() = RecentType.entries

    open fun assetFilters(): Set<AssetFilter> = emptySet()

    open fun assetsSearchLimit(query: String): Int = NO_QUERY_LIMIT

    private data class SearchRequest(
        val query: String,
        val currency: Currency,
        val chains: List<Chain>,
    )

    private companion object {
        private const val SEARCH_DEBOUNCE_MS = 250L
    }
}
