package com.gemwallet.android.application.stake.coordinators

import com.wallet.core.primitives.Delegation
import kotlinx.coroutines.flow.Flow

interface GetDelegation {
    operator fun invoke(validatorId: String, delegationId: String): Flow<Delegation?>
}
