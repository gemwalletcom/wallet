package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.HasAssetPriceAlerts
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId

class HasAssetPriceAlertsImpl(
    private val priceAlertsDao: PriceAlertsDao,
) : HasAssetPriceAlerts {

    override suspend fun invoke(assetId: AssetId): Boolean = priceAlertsDao.hasAssetPriceAlerts(assetId.toIdentifier())
}
