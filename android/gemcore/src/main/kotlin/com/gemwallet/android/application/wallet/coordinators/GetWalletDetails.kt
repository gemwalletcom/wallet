package com.gemwallet.android.application.wallet.coordinators

import com.gemwallet.android.domains.wallet.aggregates.WalletDetailsAggregate
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetWalletDetails {
    fun getWallet(walletId: WalletId): Flow<WalletDetailsAggregate?>
}
