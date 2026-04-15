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
import com.gemwallet.android.features.asset_select.viewmodels.models.SearchState
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.UIState
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
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

    private val searchState = MutableStateFlow(SearchState.Init)

    val availableChains = session
        .map { session -> session?.wallet?.accounts?.map { it.chain } ?: emptyList() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val filters = combine(
        session,
        snapshotFlow { queryState.text.toString() },
        selectedTag,
        chainFilter,
        balanceFilter,
    ) { session, query, tag, chainFilter, hasBalance ->
        SelectAssetFilters(
            session = session,
            query = query,
            tag = tag,
            chainFilter = chainFilter,
            hasBalance = hasBalance,
        )
    }.onEach { filters ->
        searchState.update { if (it != SearchState.Init) SearchState.Searching else it }
        request(filters.query, filters.tag, filters.session)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assets = combine(
        filters,
        search.items(filters),
    ) { filters, items ->
        val chainFilter = filters?.chainFilter
        val balanceFilter = filters?.hasBalance ?: false
        val hasChainFilter = chainFilter?.isNotEmpty() ?: false

        items.filter {
            (!hasChainFilter || chainFilter.contains(it.id().chain))
                    && (!balanceFilter || it.balance.totalAmount > 0.0)
        }
    }
    .map { items ->
        val wallet = session.value?.wallet
        items.map { item ->
            val info = if (item.owner == null && wallet != null) {
                item.copy(owner = wallet.getAccount(item.asset.id.chain))
            } else {
                item
            }
            AssetInfoUIModel(info)
        }
    }
//    .onEach { searchState.update { SearchState.Idle } }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>())

    val popular = assets.map { items ->
        items.filter {
            listOf(
                AssetId(Chain.Ethereum),
                AssetId(Chain.Bitcoin),
                AssetId(Chain.Solana),
            ).contains(it.asset.id)
        }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>().toImmutableList())

    val pinned = assets.map { items: List<AssetItemUIModel> ->
        items.filter { it.metadata?.isPinned == true }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>().toImmutableList())

    val unpinned = assets.map { items: List<AssetItemUIModel> ->
        items.filter { it.metadata?.isPinned != true }.toImmutableList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetItemUIModel>().toImmutableList())

    val recent = snapshotFlow { queryState.text.toString() }
        .flatMapLatest { query ->
            if (query.isNotEmpty() || !showRecents) {
                flow<List<Asset>> { emit(emptyList()) }
            } else {
                getRecentAssets(RecentAssetsRequest(filters = assetFilters()))
            }
        }
    .map { it.toImmutableList() }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<Asset>().toImmutableList())

    val uiState = assets.combine(searchState) { assets, searchState ->
        when {
            searchState != SearchState.Idle && searchState != SearchState.Init -> UIState.Loading
            assets.isEmpty() && searchState == SearchState.Idle -> UIState.Empty
            else -> UIState.Idle
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, UIState.Idle)

    val isAddAssetAvailable = getSession().map { session ->
        session?.wallet?.accounts?.any { it.chain.assetType() != null } == true
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun onChangeVisibility(assetId: AssetId, visible: Boolean) = viewModelScope.launch {
        val session = session.value ?: return@launch
        val account = session.wallet.getAccount(assetId.chain) ?: return@launch
        switchAssetVisibility(session.wallet.id, account, assetId, visible)
    }

    fun onTogglePin(assetId: AssetId) = viewModelScope.launch {
        val session = session.value ?: return@launch
        val account = session.wallet.getAccount(assetId.chain) ?: return@launch
        switchAssetVisibility(session.wallet.id, account, assetId, true)
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
        delay(250)
        searchTokensCase.search(
            query = query,
            currency = session?.currency ?: Currency.USD,
            chains = session?.wallet?.takeIf { it.type == WalletType.Multicoin }?.accounts?.map { it.chain } ?: emptyList(),
            tags = tags?.let { listOf(it) } ?: emptyList(),
        )
        searchState.update { SearchState.Idle }
    }

    fun updateRecent(assetId: AssetId, type: RecentType) = viewModelScope.launch(Dispatchers.IO) {
        val walletId = session.value?.wallet?.id ?: return@launch
        updateRecentAsset(assetId, walletId, type)
    }

    open val showRecents: Boolean get() = true

    open fun assetFilters(): Set<AssetFilter> = emptySet()
}
