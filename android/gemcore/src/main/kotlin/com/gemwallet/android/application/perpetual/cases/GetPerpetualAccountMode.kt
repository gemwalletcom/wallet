package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.WalletId

interface GetPerpetualAccountMode {
    suspend fun getPerpetualAccountMode(walletId: WalletId, address: String): PerpetualAccountMode
}
