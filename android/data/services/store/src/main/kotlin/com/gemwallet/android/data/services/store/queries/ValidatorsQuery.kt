package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.StakeDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class ValidatorsQuery @Inject constructor(private val stakeDao: StakeDao) {

    operator fun invoke(assetId: AssetId, providerType: StakeProviderType): Flow<List<DelegationValidator>> = stakeDao.getValidators(assetId, providerType).map { it.toDTO() }
}
