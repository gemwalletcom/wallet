package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualPosition
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregateImpl
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class GetPerpetualPositionImpl @Inject constructor(private val perpetualStore: GemstonePerpetualStore) : GetPerpetualPosition {
    override fun getPositionByPerpetual(walletId: WalletId, id: PerpetualId): Flow<PerpetualPosition?> = perpetualStore.observePositionByPerpetualId(walletId, id).map { it?.position }
}
