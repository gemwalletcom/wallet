package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.coordinators.GetRecommendedValidator
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationValidator
import kotlinx.coroutines.flow.Flow

class GetRecommendedValidatorImpl(
    private val stakeRepository: StakeRepository,
) : GetRecommendedValidator {
    override fun invoke(chain: Chain): Flow<DelegationValidator?> =
        stakeRepository.getRecommended(chain)
}
