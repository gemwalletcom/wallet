package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.toAssetInfoModel
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject

class AssetsQuery @Inject constructor(private val assetsDao: AssetsDao) {

    operator fun invoke(walletId: WalletId): Flow<List<AssetInfo>> = assetsDao.getAssetsInfo(walletId.id).toAssetInfoModel()
}
