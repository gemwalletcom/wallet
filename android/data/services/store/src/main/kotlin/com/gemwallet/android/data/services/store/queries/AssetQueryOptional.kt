package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class AssetQueryOptional @Inject constructor(private val assetsDao: AssetsDao) {

    operator fun invoke(walletId: String, assetId: AssetId): Flow<AssetData?> = assetsDao.getTokenInfo(walletId, assetId.toIdentifier(), assetId.chain)
        .map { it?.toDTO() }
}
