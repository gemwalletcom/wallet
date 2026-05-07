package com.gemwallet.android.features.asset_select.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.ext.assetType
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.Session
import com.gemwallet.android.ui.components.list_item.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.AssetItemUIModel
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
open class BaseAssetSelectViewModel(
    getSession: GetSession,
    private val getRecentAssets: GetRecentAssets,
    private val updateRecentAsset: UpdateRecentAsset,
    private val switchAssetVisibility: SwitchAssetVisibility,
    private val toggleAssetPin: ToggleAssetPin,
    private val searchTokensCase: SearchTokensCase,
    val search: SelectSearch,
) : ViewModel() {

    val queryState = TextFieldState()
    val selectedTag = MutableStateFlow<AssetTag?>(null)
    val chainFilter = MutableStateFlow<List<Chain>>(emptyList())
    val balanceFilter = MutableStateFlow(false)

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val noResultsQuery = MutableStateFlow<String?>(null)

    val availableChains = session
        .map { session -> session?.wallet?.accounts?.map { it.chain } ?: emptyList() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val currentQuery = snapshotFlow { queryState.text.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val filters = combine(
        session,
        currentQuery,
        selectedTag,
        chainFilter,
        balanceFilter,
    ) { session, query, tag, chainFilter, hasBalance ->
        SelectAssetFilters(session = session, query = query, chainFilter = chainFilter, hasBalance = hasBalance, tag = tag)
    }.onEach { request(it.query, it.tag, it.session) }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assets = combine(
        filters,
        search.items(filters),
        noResultsQuery,
    ) { filters, items, noResultsQuery ->
        val query = filters?.query.orEmpty()
        if (query.isNotEmpty() && noResultsQuery != null) return@combine emptyList()

        val chainFilter = filters?.chainFilter.orEmpty()
        val balanceFilter = filters?.hasBalance == true
        val wallet = session.value?.wallet
        items
            .filter { (chainFilter.isEmpty() || it.id().chain in chainFilter) && (!balanceFilter || it.balance.totalAmount > 0.0) }
            .map { item ->
                val owner = item.owner ?: wallet?.getAccount(item.asset.id.chain)
                AssetInfoUIModel(if (item.owner == owner) item else item.copy(owner = owner))
            }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>())

    val popular = assets.map { items ->
        items.filter {
            it.asset.id in listOf(AssetId(Chain.Ethereum), AssetId(Chain.Bitcoin), AssetId(Chain.Solana))
        }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>().toImmutableList())

    val pinned = assets.map { items ->
        items.filter { it.metadata?.isPinned == true && it.metadata?.isBalanceEnabled == true }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>().toImmutableList())

    val unpinned = assets.map { items ->
        items.filter { it.metadata?.isPinned != true || it.metadata?.isBalanceEnabled != true }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>().toImmutableList())

    val recent = currentQuery
        .flatMapLatest { query ->
            if (query.isNotEmpty() || !showRecents) {
                flow { emit(emptyList()) }
            } else {
                getRecentAssets(RecentAssetsRequest(filters = assetFilters()))
            }
        }
    .map { items -> items.map { it.asset }.toImmutableList() }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<Asset>().toImmutableList())

    val uiState = combine(assets, currentQuery, noResultsQuery) { assets, query, noResultsQuery ->
        when {
            assets.isNotEmpty() -> UIState.Idle
            query.isNotEmpty() && query == noResultsQuery -> UIState.Empty
            query.isNotEmpty() -> UIState.Loading
            else -> UIState.Idle
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Idle)

    val isAddAssetAvailable = getSession().map { session ->
        session?.wallet?.accounts?.any { it.chain.assetType() != null } == true
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun onChangeVisibility(assetId: AssetId, visible: Boolean) = viewModelScope.launch {
        val session = session.value ?: return@launch
        session.wallet.getAccount(assetId.chain) ?: return@launch
        switchAssetVisibility(session.wallet.id, assetId, visible)
    }

    fun onTogglePin(assetId: AssetId) = viewModelScope.launch {
        val session = session.value ?: return@launch
        session.wallet.getAccount(assetId.chain) ?: return@launch
        toggleAssetPin(session.wallet.id, assetId)
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

    fun onTagSelect(tag: AssetTag?) {
        selectedTag.update { tag }
    }

    open fun getTags(): List<AssetTag?> = listOf(
        null,
        AssetTag.Trending,
        AssetTag.Stablecoins,
    )

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

    private fun request(query: String, tags: AssetTag?, session: Session?) = viewModelScope.launch(Dispatchers.IO) {
        delay(SEARCH_DEBOUNCE_MS)
        val ok = searchTokensCase.search(
            query = query,
            currency = session?.currency ?: Currency.USD,
            chains = walletSearchChains(session?.wallet),
            tags = tags?.let { listOf(it) } ?: emptyList(),
        )
        noResultsQuery.value = if (ok) null else query
    }

    private fun walletSearchChains(wallet: Wallet?): List<Chain> = when (wallet?.type) {
        WalletType.Multicoin -> emptyList()
        WalletType.Single, WalletType.PrivateKey, WalletType.View -> listOfNotNull(wallet.accounts.firstOrNull()?.chain)
        null -> emptyList()
    }

    fun updateRecent(assetId: AssetId, type: RecentType) = viewModelScope.launch(Dispatchers.IO) {
        val walletId = session.value?.wallet?.id ?: return@launch
        updateRecentAsset(assetId, walletId, type)
    }

    open val showRecents: Boolean get() = true

    open fun assetFilters(): Set<AssetFilter> = emptySet()

    private companion object {
        private const val SEARCH_DEBOUNCE_MS = 250L
    }
}
