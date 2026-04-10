package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetTag
import kotlinx.coroutines.flow.Flow

interface SearchSelectAssets {
    operator fun invoke(query: String, tags: List<AssetTag>): Flow<List<AssetInfo>>
}
