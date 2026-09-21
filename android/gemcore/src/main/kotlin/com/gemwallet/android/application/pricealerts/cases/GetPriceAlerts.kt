package com.gemwallet.android.application.pricealerts.cases

import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import kotlinx.coroutines.flow.Flow

interface GetPriceAlerts {
    operator fun invoke(assetId: AssetId? = null): Flow<List<PriceAlertDataAggregate>>

    fun assetPriceAlerts(assetId: AssetId): Flow<List<PriceAlert>>
}
