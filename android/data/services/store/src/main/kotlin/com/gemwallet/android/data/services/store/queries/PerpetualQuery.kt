package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.PerpetualDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualQuery @Inject constructor(private val perpetualDao: PerpetualDao, private val assetsDao: AssetsDao) {

    operator fun invoke(assetId: AssetId): Flow<PerpetualData?> = perpetualDao.getPerpetualByAssetId(assetId.toIdentifier()).map { it?.toDTO() }

    operator fun invoke(perpetualId: PerpetualId): Flow<PerpetualData?> = perpetualDao.getPerpetual(perpetualId.toIdentifier()).map { it?.toDTO() }

    fun associatedAssets(assetId: AssetId): Flow<List<Asset>> = perpetualDao.getPerpetualByAssetId(assetId.toIdentifier())
        .map { perpetual -> perpetual?.asset?.associations.orEmpty().filter { it.type == AssetAssociationType.Official }.map { it.assetId.toIdentifier() } }
        .distinctUntilChanged()
        .flatMapLatest { assetIds -> assetsDao.getAssets(assetIds) }
        .map { assets -> assets.mapNotNull { it.toDTO() } }
}
