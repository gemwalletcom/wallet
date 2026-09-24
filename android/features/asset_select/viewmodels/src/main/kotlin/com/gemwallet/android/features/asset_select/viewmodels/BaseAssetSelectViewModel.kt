package com.gemwallet.android.features.asset_select.viewmodels

import android.content.Context
import android.util.Log
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.domains.asset.assetSections
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset_select.viewmodels.models.AssetSelectFlowUIModel
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.uiModel
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.assetAddedToast
import com.gemwallet.android.ui.components.screen.assetPinnedToast
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
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
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemSelectAssetState
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.addressCopy

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
open class BaseAssetSelectViewModel(
    getSession: GetSession,
    private val recentAssetsService: RecentAssetsService,
    protected val service: GemAssetSelectionServiceInterface,
    val search: SelectSearch,
    selectType: GemSelectAssetType,
    protected val ioDispatcher: CoroutineDispatcher,
    protected val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    val flow = service.flow(selectType)

    val flowUIModel: AssetSelectFlowUIModel = flow.uiModel(context)

    fun reset() {
        queryState.clearText()
        chainFilter.value = emptyList()
        balanceFilter.value = false
    }

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val isSearching = MutableStateFlow(false)

    val queryState = TextFieldState()
    val chainFilter = MutableStateFlow<List<Chain>>(emptyList())
    val balanceFilter = MutableStateFlow(false)

    private val walletFlow = session
        .map { session -> session?.wallet?.let { service.walletFlow(selectType, it.toGem()) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableChains = walletFlow
        .map { walletFlow -> walletFlow?.chains?.map { chain -> chain.requireChain() }.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    protected val currentQuery = snapshotFlow { queryState.text.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val searchRequests = currentQuery.debounce(service.searchDebounceMilliseconds().toLong()).distinctUntilChanged()

    private val filters = combine(
        session,
        currentQuery,
        chainFilter,
        balanceFilter,
    ) { session, query, chainFilter, hasBalance ->
        SelectAssetFilters(
            session = session,
            query = query,
            limit = assetsSearchLimit(query),
            scope = flow.scope,
            filters = flow.appliedFilters(chainFilter.map { it.string }, hasBalance),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetsContent = combine(
        filters,
        search.items(filters),
    ) { _, items ->
        val wallet = session.value?.wallet
        items
            .map { item ->
                val owner = item.owner ?: wallet?.getAccount(item.asset.id.chain)
                if (item.owner == owner) item else item.copy(owner = owner)
            }
            .toAssetInfoDataAggregates(flow.rowStyle)
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

    val recent = combine(currentQuery, chainFilter, balanceFilter) { query, chains, hasBalance -> query to assetFilters(chains, hasBalance) }
        .flatMapLatest { (query, filters) ->
            if (query.isNotEmpty() || !flow.recents) {
                flow { emit(emptyList()) }
            } else {
                recentAssetsService.getRecentAssets(RecentAssetsRequest(types = recentTypes, filters = filters))
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

    fun onSelected(asset: Asset) {
        flow.action?.let { updateRecent(asset, it) }
        if (flow.enablesPriceAlert) {
            viewModelScope.launch(ioDispatcher) {
                runCatchingCancellable { service.setPriceAlert(asset.id.toIdentifier(), true) }
                    .onFailure { Log.e(TAG, "enabling the price alert for ${asset.id.toIdentifier()} failed", it) }
            }
        }
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
        val item = assets.value.firstOrNull { it.asset.id == assetId }
        val willPin = item?.pinned != true
        runCatchingCancellable { service.setAssetPinned(assetId.toIdentifier(), willPin) }
            .onSuccess { item?.let { emitToast(assetPinnedToast(context, it.asset.name, willPin)) } }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
    }

    private suspend fun setVisibility(assetId: AssetId, visible: Boolean): Result<Unit> = withContext(ioDispatcher) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), visible) }
            .onFailure { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
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

    protected suspend fun setPerpetualPinned(perpetualId: PerpetualId, pinned: Boolean): Result<Unit> = withContext(ioDispatcher) {
        runCatchingCancellable { service.setPerpetualPinned(perpetualId.toIdentifier(), pinned) }
            .onFailure { Log.e(TAG, "pinning perpetual ${perpetualId.toIdentifier()} failed", it) }
    }

    fun openRecent(asset: Asset) = updateRecent(asset, GemAssetAction.OPEN)

    fun updateRecent(asset: Asset, action: GemAssetAction) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.addRecent(action, asset.toGem()) }
            .onFailure { Log.e(TAG, "recording recent ${asset.id.toIdentifier()} failed", it) }
    }

    val recentTypes: List<RecentActivityType>
        get() = flow.action?.recentActivityTypes()?.map { it.toPrimitives() } ?: RecentActivityType.entries

    fun assetFilters(): Set<GemAssetFilter> = assetFilters(chainFilter.value, balanceFilter.value)

    private fun assetFilters(chains: List<Chain>, hasBalance: Boolean): Set<GemAssetFilter> = flow.appliedFilters(chains.map { it.string }, hasBalance).toSet()

    open fun assetsSearchLimit(query: String): Int = ASSETS_LIMIT

    private companion object {
        private const val TAG = "AssetSelect"
        private const val ASSETS_LIMIT = 100
    }
}
