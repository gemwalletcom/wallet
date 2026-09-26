package com.gemwallet.android.features.assets.viewmodels.select.models

import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

interface SelectSearch {
    fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>>
}
