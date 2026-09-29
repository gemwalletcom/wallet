package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.data.services.store.queries.PerpetualsQuery
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.data.services.store.queries.WalletSearchQuery
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.assets.viewmodels.select.BaseSelectAssetViewModel
import com.gemwallet.android.features.assets.viewmodels.select.models.BaseSelectSearch
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.Wallet
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemNftEntry
import uniffi.gemstone.GemPerpetualMarketItem
import uniffi.gemstone.GemSearchListRow
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemSelectAssetState
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemWalletSearchInput
import uniffi.gemstone.GemWalletSearchView
import uniffi.gemstone.perpetualMarketQuery
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WalletSearchViewModel @Inject constructor(
    getSession: GetSession,
    assetsQuery: AssetsQuery,
    walletSearchQuery: WalletSearchQuery,
    recentActivityQuery: RecentActivityQuery,
    service: GemAssetSelectionServiceInterface,
    perpetualsQuery: PerpetualsQuery,
    nftQuery: NFTQuery,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
) : BaseSelectAssetViewModel(
    getSession,
    recentActivityQuery,
    service,
    BaseSelectSearch(assetsQuery),
    GemSelectAssetType.WalletSearch,
    ioDispatcher,
    context,
) {

    override suspend fun searchRemote(query: String) {
        service.search(query, GemSearchScope.All)
    }

    private val perpetuals: Flow<List<PerpetualData>> = currentQuery
        .flatMapLatest { query -> perpetualMarketQuery(query).let { perpetualsQuery(it.search, it.limit.toInt(), it.requiresVolume) } }

    private val collections: Flow<List<NFTData>> = getSession()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { nftQuery(it.wallet.id.id) }

    private val searchLists: Flow<List<AssetList>> = currentQuery
        .flatMapLatest { query -> walletSearchQuery.lists(query) }

    private val rows: Flow<WalletSearchRows> = combine(pinned, unpinned, perpetuals, searchLists, collections, ::WalletSearchRows)

    private val content: StateFlow<SearchContent<GemWalletSearchView>?> = combine(getSession(), currentQuery, searching, recent, rows) { session, query, searching, recents, rows ->
        session?.wallet?.let { wallet ->
            SearchContent(service.walletSearchView(rows.input(wallet, query, searching, recents.size)), rows.assets)
        }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val state: StateFlow<GemSelectAssetState> = content.select(viewModelScope, GemSelectAssetState.IDLE) { it.view.state.phase }
    val pinnedAssets: StateFlow<List<AssetInfoDataAggregate>> = content.select(viewModelScope, emptyList()) { it.assets(it.view.pinnedAssetIds) }
    val previewAssets: StateFlow<List<AssetInfoDataAggregate>> = content.select(viewModelScope, emptyList()) { it.assets(it.view.assetIds) }
    val hasMoreAssets: StateFlow<Boolean> = content.select(viewModelScope, false) { it.view.hasMoreAssets }
    val pinnedPerpetuals: StateFlow<List<GemPerpetualMarketItem>> = content.select(viewModelScope, emptyList()) { it.view.pinnedPerpetuals }
    val previewPerpetuals: StateFlow<List<GemPerpetualMarketItem>> = content.select(viewModelScope, emptyList()) { it.view.perpetuals }
    val hasMorePerpetuals: StateFlow<Boolean> = content.select(viewModelScope, false) { it.view.hasMorePerpetuals }
    val lists: StateFlow<List<GemSearchListRow>> = content.select(viewModelScope, emptyList()) { it.view.lists }
    val previewNfts: StateFlow<List<GemNftEntry>> = content.select(viewModelScope, emptyList()) { it.view.nfts }
    val hasMoreNfts: StateFlow<Boolean> = content.select(viewModelScope, false) { it.view.hasMoreNfts }

    override fun assetsSearchLimit(query: String): Int = service.walletSearchLimits(query).fetch.toInt()
}

private class WalletSearchRows(
    val pinned: List<AssetInfoDataAggregate>,
    unpinned: List<AssetInfoDataAggregate>,
    val perpetuals: List<PerpetualData>,
    val lists: List<AssetList>,
    val collections: List<NFTData>,
) {
    val assets: List<AssetInfoDataAggregate> = pinned + unpinned

    fun input(wallet: Wallet, query: String, isLoading: Boolean, recents: Int) = GemWalletSearchInput(
        wallet = wallet.toGem(),
        query = query,
        isLoading = isLoading,
        recents = recents.toUInt(),
        assetIds = assets.map { it.asset.id.toIdentifier() },
        pinnedAssetIds = pinned.map { it.asset.id.toIdentifier() },
        perpetuals = perpetuals.map { it.toGem() },
        lists = lists.map { it.toGem() },
        collections = collections.map { it.toGem() },
    )
}
