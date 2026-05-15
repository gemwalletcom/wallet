package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.coordinators.GetDelegations
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

class GetDelegationsImpl(
    private val stakeRepository: StakeRepository,
) : GetDelegations {
    override fun invoke(walletId: WalletId, assetId: AssetId): Flow<List<Delegation>> =
        stakeRepository.getDelegations(walletId, assetId)
}
