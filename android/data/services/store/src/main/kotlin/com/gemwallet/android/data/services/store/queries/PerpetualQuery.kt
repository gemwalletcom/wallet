package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.PerpetualDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class PerpetualQuery @Inject constructor(private val perpetualDao: PerpetualDao) {

    operator fun invoke(assetId: AssetId): Flow<PerpetualData?> = perpetualDao.getPerpetualByAssetId(assetId.toIdentifier()).map { it?.toDTO() }

    operator fun invoke(perpetualId: PerpetualId): Flow<PerpetualData?> = perpetualDao.getPerpetual(perpetualId.toIdentifier()).map { it?.toDTO() }
}
