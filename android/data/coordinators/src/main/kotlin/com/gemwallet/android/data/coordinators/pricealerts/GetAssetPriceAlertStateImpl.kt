package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.GetAssetPriceAlertState
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetAssetPriceAlertStateImpl(
    private val priceAlertsDao: PriceAlertsDao,
) : GetAssetPriceAlertState {

    override fun isAssetPriceAlertEnabled(assetId: AssetId): Flow<Boolean> =
        priceAlertsDao.getAssetPriceAlert(assetId.toIdentifier())
            .mapLatest { it?.toDTO()?.priceAlert != null }
            .flowOn(Dispatchers.IO)
}
