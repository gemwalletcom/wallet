package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.PerpetualPositionDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class PerpetualPositionsQuery @Inject constructor(private val perpetualPositionDao: PerpetualPositionDao) {

    operator fun invoke(walletId: WalletId): Flow<List<PerpetualPositionData>> = perpetualPositionDao.getPositionsData(walletId.id).map { items -> items.mapNotNull { it.toDTO() } }

    operator fun invoke(walletId: WalletId, perpetualId: PerpetualId): Flow<PerpetualPositionData?> = perpetualPositionDao.getPositionDataByPerpetual(walletId.id, perpetualId.toIdentifier()).map { it?.toDTO() }
}
