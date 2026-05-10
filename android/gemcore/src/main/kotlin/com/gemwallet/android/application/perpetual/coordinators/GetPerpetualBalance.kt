package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetPerpetualBalance {
    fun getBalance(walletId: WalletId): Flow<PerpetualBalance?>
}
