package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

class GetAssetPriceUsdImpl(
    private val pricesDao: PricesDao,
) : GetAssetPriceUsd {

    override fun invoke(assetId: AssetId): Flow<Double?> = pricesDao.getUsdPrice(assetId.toIdentifier())
}
