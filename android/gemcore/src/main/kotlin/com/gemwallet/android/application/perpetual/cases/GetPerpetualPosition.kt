package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetPerpetualPosition {
    fun getPositionByPerpetual(walletId: WalletId, id: PerpetualId): Flow<PerpetualPosition?>
}
