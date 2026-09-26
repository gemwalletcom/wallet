package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.StakeDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.DelegationValidator
import javax.inject.Inject

class ValidatorQuery @Inject constructor(private val stakeDao: StakeDao) {

    suspend operator fun invoke(assetId: AssetId, validatorId: String): DelegationValidator? = stakeDao.getValidator(assetId, validatorId)?.toDTO()
}
