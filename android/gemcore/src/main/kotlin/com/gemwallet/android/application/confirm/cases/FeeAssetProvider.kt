package com.gemwallet.android.application.confirm.cases

import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

interface FeeAssetProvider {
    fun getFeeAssets(): Flow<List<AssetInfo>>
}
