package com.gemwallet.android.application.perpetual.coordinators

import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow

interface GetPerpetual {
    fun getPerpetual(perpetualId: String): Flow<PerpetualDetailsDataAggregate?>

    fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualDetailsDataAggregate?>
}
