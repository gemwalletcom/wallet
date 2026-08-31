package com.gemwallet.android.application.pricealerts.cases

import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetAssetPriceAlertState {
    fun isAssetPriceAlertEnabled(assetId: AssetId): Flow<Boolean>
}