package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.PricesDao
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject

class PriceUsdQuery @Inject constructor(private val pricesDao: PricesDao) {

    operator fun invoke(assetId: AssetId): Flow<Double?> = pricesDao.getUsdPrice(assetId.toIdentifier())
}
