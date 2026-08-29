package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.Flow

class GetSelectAssetsInfoImpl(
    private val getWalletAssets: GetWalletAssets,
) : GetSelectAssetsInfo {
    override fun invoke(): Flow<List<AssetInfo>> = getWalletAssets()
}
