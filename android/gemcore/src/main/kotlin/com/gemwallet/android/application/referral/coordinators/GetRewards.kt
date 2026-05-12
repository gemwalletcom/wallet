package com.gemwallet.android.application.referral.coordinators

import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.WalletId

interface GetRewards {
    suspend fun getRewards(walletId: WalletId): Rewards
}
