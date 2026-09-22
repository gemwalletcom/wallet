package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject

class GetPerpetualImpl @Inject constructor(private val perpetualStore: GemstonePerpetualStore) : GetPerpetual {

    override fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualData?> = perpetualStore.observePerpetual(perpetualId)

    override fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?> = perpetualStore.observePerpetualByAssetId(assetId)
}
