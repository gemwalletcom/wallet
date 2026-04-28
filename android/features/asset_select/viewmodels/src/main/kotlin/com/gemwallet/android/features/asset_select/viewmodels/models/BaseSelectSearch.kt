package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
open class BaseSelectSearch(
    private val searchSelectAssets: SearchSelectAssets,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters.flatMapLatest { filters ->
            searchSelectAssets(filters?.query ?: "", filters?.tag?.let { listOf(it) } ?: emptyList())
        }
    }
}
