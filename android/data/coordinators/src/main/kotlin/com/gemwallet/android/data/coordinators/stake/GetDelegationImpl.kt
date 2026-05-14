package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.coordinators.GetDelegation
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.wallet.core.primitives.Delegation
import kotlinx.coroutines.flow.Flow

class GetDelegationImpl(
    private val stakeRepository: StakeRepository,
) : GetDelegation {
    override fun invoke(validatorId: String, delegationId: String): Flow<Delegation?> =
        stakeRepository.getDelegation(validatorId = validatorId, delegationId = delegationId)
}
