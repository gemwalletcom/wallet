package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.application.assets.values.AssetsQueryScope
import com.gemwallet.android.application.assets.values.toQueryFilter
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import uniffi.gemstone.GemSelectAssetScope

@OptIn(ExperimentalCoroutinesApi::class)
class BaseSelectSearch(private val assetsQuery: AssetsQuery) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> = filters.flatMapLatest { filters ->
        val walletId = filters?.session?.wallet?.id ?: return@flatMapLatest emptyFlow()
        assetsQuery(
            walletId = walletId,
            searchBy = filters.query,
            scope = when (filters.scope) {
                GemSelectAssetScope.WALLET -> AssetsQueryScope.Wallet
                GemSelectAssetScope.ALL_ASSETS -> AssetsQueryScope.AllAssets
            },
            filters = filters.queryFilters().map { it.toQueryFilter() }.toSet(),
            limit = filters.limit,
        )
    }
}
