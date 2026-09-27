package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.PricesDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class PriceQuery @Inject constructor(private val pricesDao: PricesDao) {

    operator fun invoke(assetId: AssetId): Flow<PriceData?> = pricesDao.getPriceInfo(assetId.toIdentifier()).map { it?.toDTO() }
}
