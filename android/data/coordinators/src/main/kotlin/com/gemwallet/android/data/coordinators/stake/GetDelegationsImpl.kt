package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.coordinators.GetDelegations
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import kotlinx.coroutines.flow.Flow

class GetDelegationsImpl(
    private val stakeRepository: StakeRepository,
) : GetDelegations {
    override fun invoke(assetId: AssetId, owner: String): Flow<List<Delegation>> =
        stakeRepository.getDelegations(assetId, owner)
}
