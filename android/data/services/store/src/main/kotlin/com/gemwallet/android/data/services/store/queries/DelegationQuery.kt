package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.StakeDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class DelegationQuery @Inject constructor(private val stakeDao: StakeDao) {

    operator fun invoke(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?> = stakeDao.getDelegation(walletId, validatorId, delegationId).map { it?.toModel() }
}
