package com.gemwallet.android.features.assets.viewmodels.select

import android.content.Context
import android.util.Log
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.values.toQueryFilter
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.domains.asset.assetSections
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.assets.viewmodels.select.models.SelectAssetFilters
import com.gemwallet.android.features.assets.viewmodels.select.models.SelectSearch
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.assetAddedToast
import com.gemwallet.android.ui.components.screen.message
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.RecentActivityType
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetAction
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemAssetSearchStep
import uniffi.gemstone.GemAssetSectionCounts
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemAssetsFilterView
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemEmptyState
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemSelectAssetState
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemToast
import uniffi.gemstone.addressCopy
import uniffi.gemstone.emptyState

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
open class BaseSelectAssetViewModel(
    getSession: GetSession,
    private val recentActivityQuery: RecentActivityQuery,
    protected val service: GemAssetSelectionServiceInterface,
    val search: SelectSearch,
    selectType: GemSelectAssetType,
    protected val ioDispatcher: CoroutineDispatcher,
    protected val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    val flow = service.flow(selectType)

    fun reset() {
        queryState.clearText()
        filterSession.update { it.onClear() }
    }

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val walletId = session
        .filterNotNull()
        .map { it.wallet.id }
        .distinctUntilChanged()

    private val isSearching = MutableStateFlow(false)

    val queryState = TextFieldState()
    private val filterSession = MutableStateFlow(flow.filterSession(emptyList()))
    val filterView: StateFlow<GemAssetsFilterView> = filterSession
        .map { it.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, filterSession.value.viewState())

    private val walletFlow = session
        .map { session -> session?.wallet?.let { service.walletFlow(selectType, it.toGem()) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableChains = walletFlow
        .map { walletFlow -> walletFlow?.chains?.map { chain -> chain.requireChain() }.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    protected val currentQuery = snapshotFlow { queryState.text.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val searchRequests = currentQuery.debounce(GemConstants.searchDebounce).distinctUntilChanged()

    private val filters = combine(
        session,
        currentQuery,
        filterView,
    ) { session, query, filterView ->
        SelectAssetFilters(
            session = session,
            query = query,
            limit = assetsSearchLimit(query),
            scope = flow.scope,
            filters = filterView.filters,
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetsContent = combine(
        filters,
        search.items(filters),
    ) { _, items ->
        val current = session.value
        items
            .map { item ->
                val account = item.account.takeIf { it.address.isNotEmpty() } ?: current?.wallet?.getAccount(item.asset.id.chain)
                if (account == null || account == item.account) item else item.copy(account = account)
            }
            .toAssetInfoDataAggregates(current?.currency ?: Currency.USD, flow.rowStyle)
    }
        .flowOn(ioDispatcher)
        .shareIn(viewModelScope, SharingStarted.Eagerly, replay = 1)

    private data class AssetSections(
        val popular: ImmutableList<AssetInfoDataAggregate> = emptyList<AssetInfoDataAggregate>().toImmutableList(),
        val pinned: ImmutableList<AssetInfoDataAggregate> = emptyList<AssetInfoDataAggregate>().toImmutableList(),
        val unpinned: ImmutableList<AssetInfoDataAggregate> = emptyList<AssetInfoDataAggregate>().toImmutableList(),
    )

    private fun assetSections(items: List<AssetInfoDataAggregate>): AssetSections {
        val sections = items.assetSections(
            showsPopular = flow.popularSection,
            assetId = { it.asset.id },
            isPinned = { it.pinned },
        )
        return AssetSections(
            popular = sections.popular.toImmutableList(),
            pinned = sections.pinned.toImmutableList(),
            unpinned = sections.unpinned.toImmutableList(),
        )
    }

    private val assets = assetsContent
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<AssetInfoDataAggregate>())

    private val sections = assets
        .map(::assetSections)
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetSections(assets.value))

    val popular = sections
        .map { it.popular }
        .stateIn(viewModelScope, SharingStarted.Eagerly, sections.value.popular)

    val pinned = sections
        .map { it.pinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, sections.value.pinned)

    val unpinned = sections
        .map { it.unpinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, sections.value.unpinned)

    val recent = combine(currentQuery, filterView) { query, filterView -> query to filterView.filters.toSet() }
        .flatMapLatest { (query, filters) ->
            if (query.isNotEmpty() || !flow.recents) {
                flow { emit(emptyList()) }
            } else {
                walletId.flatMapLatest { recentActivityQuery(it, recentTypes, filters.map { filter -> filter.toQueryFilter() }.toSet(), GemConstants.recentAssetsLimit) }
            }
        }
        .map { items -> items.map { it.asset }.toImmutableList() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList<Asset>().toImmutableList())

    val showsRecents: StateFlow<Boolean> = combine(snapshotFlow { queryState.text.isNotEmpty() }, recent) { hasQuery, recents -> flow.showsRecents(hasQuery, recents.isNotEmpty()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val uiState = combine(sections, isSearching) { sections, isSearching ->
        val counts = GemAssetSectionCounts(
            pinned = sections.pinned.size.toUInt(),
            popular = sections.popular.size.toUInt(),
            assets = sections.unpinned.size.toUInt(),
        )
        flow.state(counts, isSearching)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemSelectAssetState.IDLE)

    val isChainFilterAvailable = walletFlow
        .map { it?.showsChainFilter == true }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val isAddAssetAvailable = walletFlow
        .map { it?.showsAddToken == true }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val searchEmptyState: StateFlow<GemEmptyState> = walletFlow
        .map { it?.emptyState ?: emptyState(GemEmptyStateKind.SEARCH_ASSETS) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyState(GemEmptyStateKind.SEARCH_ASSETS))

    fun onSelected(asset: Asset) {
        flow.action?.let { updateRecent(asset, it) }
    }

    fun addressCopy(item: AssetInfoDataAggregate): GemCopy = addressCopy(item.asset.id.chain.string, item.accountAddress)

    fun onChangeVisibility(assetId: AssetId, visible: Boolean) = viewModelScope.launch {
        setVisibility(assetId, visible)
    }

    fun onAddToWallet(assetId: AssetId) = viewModelScope.launch {
        if (setVisibility(assetId, visible = true).isSuccess) {
            emitToast(assetAddedToast(context))
        }
    }

    fun onTogglePin(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        val item = assets.value.firstOrNull { it.asset.id == assetId } ?: return@launch
        runCatchingCancellable { service.setAssetPinned(item.asset.toGem(), !item.pinned) }
            .onSuccess { emitToast(it.message(context)) }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
    }

    private suspend fun setVisibility(assetId: AssetId, visible: Boolean): Result<Unit> = withContext(ioDispatcher) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), visible) }
            .onFailure { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
    }

    fun setChainFilter(chains: List<Chain>) {
        filterSession.update { it.onChains(chains.map { chain -> chain.string }) }
    }

    fun onChainFilter(chain: Chain) {
        filterSession.update { it.onChainToggled(chain.string) }
    }

    fun onBalanceFilter(onlyWithBalance: Boolean) {
        filterSession.update { it.onBalance(onlyWithBalance) }
    }

    fun onClearFilters() {
        filterSession.update { it.onClear() }
    }

    init {
        if (flow.networkSearch) {
            viewModelScope.launch(ioDispatcher) {
                searchRequests.collectLatest { input ->
                    val step = flow.searchStep(input)
                    if (step !is GemAssetSearchStep.Search) return@collectLatest
                    isSearching.value = true
                    try {
                        runCatchingCancellable { searchRemote(step.query) }
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

    protected suspend fun setPerpetualPinned(perpetualId: PerpetualId, name: String, pinned: Boolean): Result<GemToast> = withContext(ioDispatcher) {
        runCatchingCancellable { service.setPerpetualPinned(perpetualId.toIdentifier(), name, pinned) }
            .onFailure { Log.e(TAG, "pinning perpetual ${perpetualId.toIdentifier()} failed", it) }
    }

    fun openRecent(asset: Asset) = updateRecent(asset, GemAssetAction.OPEN)

    fun updateRecent(asset: Asset, action: GemAssetAction) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.addRecent(action, asset.toGem()) }
            .onFailure { Log.e(TAG, "recording recent ${asset.id.toIdentifier()} failed", it) }
    }

    val recentTypes: List<RecentActivityType>
        get() = flow.action?.recentActivityTypes()?.map { it.toPrimitives() } ?: RecentActivityType.entries

    fun assetFilters(): Set<GemAssetFilter> = filterView.value.filters.toSet()

    open fun assetsSearchLimit(query: String): Int = GemConstants.assetResultsLimit

    private companion object {
        private const val TAG = "AssetSelect"
    }
}
