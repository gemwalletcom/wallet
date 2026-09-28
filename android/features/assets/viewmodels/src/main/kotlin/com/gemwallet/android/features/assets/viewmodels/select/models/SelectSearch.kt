package com.gemwallet.android.features.assets.viewmodels.select.models

import com.wallet.core.primitives.AssetData
import kotlinx.coroutines.flow.Flow

interface SelectSearch {
    fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetData>>
}
