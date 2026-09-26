package com.gemwallet.android.features.assets.viewmodels.select.models

import com.gemwallet.android.application.assets.values.toQueryFilter
import com.gemwallet.android.data.services.store.queries.WalletSearchQuery
import com.wallet.core.primitives.AssetData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class ListSelectSearch(private val walletSearchQuery: WalletSearchQuery, private val searchKey: String) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetData>> = filters.flatMapLatest { filters ->
        val walletId = filters?.session?.wallet?.id ?: return@flatMapLatest emptyFlow()
        walletSearchQuery.assets(walletId, searchKey, filters.queryFilters().map { it.toQueryFilter() }.toSet(), filters.limit)
    }
}
