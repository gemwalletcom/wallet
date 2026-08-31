package com.gemwallet.android.application.perpetual.cases

import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetPerpetualPosition {
    fun getPositionByPerpetual(walletId: WalletId, id: PerpetualId): Flow<PerpetualPositionDetailsDataAggregate?>
}
