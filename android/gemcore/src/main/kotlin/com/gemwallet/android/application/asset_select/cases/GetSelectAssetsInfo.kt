package com.gemwallet.android.application.asset_select.cases

import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

interface GetSelectAssetsInfo {
    operator fun invoke(): Flow<List<AssetInfo>>
}
