package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

interface GetSelectAssetsInfo {
    fun getAssetsInfo(): Flow<List<AssetInfo>>
}
