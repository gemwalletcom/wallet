package com.gemwallet.android.application.pricealerts.cases

import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import kotlinx.coroutines.flow.Flow

interface GetPriceAlerts {
    operator fun invoke(assetId: AssetId? = null): Flow<List<PriceAlertDataAggregate>>

    fun assetPriceAlerts(assetId: AssetId): Flow<List<PriceAlert>>

    fun groupByTargetAndAsset(items: List<PriceAlertDataAggregate>): Map<AssetId?, List<PriceAlertDataAggregate>> {
        val result = mutableMapOf<AssetId?, List<PriceAlertDataAggregate>>()

        val withoutTarget = items.filter { !it.kind.groupsByAsset() }
        val withTarget = (items - withoutTarget.toSet()).groupBy { it.assetId }

        result[null] = withoutTarget
        result.putAll(withTarget)

        return result
    }
}