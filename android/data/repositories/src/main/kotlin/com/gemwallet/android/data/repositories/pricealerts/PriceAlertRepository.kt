package com.gemwallet.android.data.repositories.pricealerts

import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface PriceAlertRepository {
    suspend fun hasAssetPriceAlerts(assetId: AssetId): Boolean


    fun getPriceAlerts(assetId: AssetId? = null): Flow<List<PriceAlertInfo>>

    fun getPriceAlertAssetIds(): Flow<List<AssetId>>

    fun getAssetPriceAlert(assetId: AssetId): Flow<PriceAlertInfo?>


    suspend fun getPriceAlert(priceAlertId: Int): PriceAlertInfo?


}
