package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.flow.Flow

interface GetPerpetual {
    fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualData?>

    fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?>
}
