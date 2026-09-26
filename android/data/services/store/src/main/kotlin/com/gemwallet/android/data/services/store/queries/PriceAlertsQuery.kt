package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.PriceAlertsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlertData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class PriceAlertsQuery @Inject constructor(private val priceAlertsDao: PriceAlertsDao) {

    operator fun invoke(assetId: AssetId? = null): Flow<List<PriceAlertData>> = (assetId?.let { priceAlertsDao.getAlertsWithAsset(it.toIdentifier()) } ?: priceAlertsDao.getAlertsWithAsset())
        .map { rows -> rows.mapNotNull { it.toDTO() } }
}
