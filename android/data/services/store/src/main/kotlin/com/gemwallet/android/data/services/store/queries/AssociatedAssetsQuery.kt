package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class AssociatedAssetsQuery @Inject constructor(private val assetsDao: AssetsDao) {

    operator fun invoke(assetId: AssetId, type: AssetAssociationType): Flow<List<Asset>> = assetsDao.getAsset(assetId.toIdentifier())
        .map { asset -> asset?.associations.orEmpty().filter { it.type == type }.map { it.assetId.toIdentifier() } }
        .distinctUntilChanged()
        .flatMapLatest { assetIds -> assetsDao.getAssets(assetIds) }
        .map { assets -> assets.mapNotNull { it.toDTO() } }
}
