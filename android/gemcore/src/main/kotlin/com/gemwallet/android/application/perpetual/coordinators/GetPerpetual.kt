package com.gemwallet.android.application.perpetual.coordinators

import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.flow.Flow

interface GetPerpetual {
    fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualDetailsDataAggregate?>

    fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualDetailsDataAggregate?>
}
