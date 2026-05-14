package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.coordinators.GetStakeValidator
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.DelegationValidator

class GetStakeValidatorImpl(
    private val stakeRepository: StakeRepository,
) : GetStakeValidator {
    override suspend fun invoke(assetId: AssetId, validatorId: String): DelegationValidator? =
        stakeRepository.getStakeValidator(assetId, validatorId)
}
