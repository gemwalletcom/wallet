package com.gemwallet.android.application.stake.coordinators

import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetDelegation {
    operator fun invoke(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?>
}
