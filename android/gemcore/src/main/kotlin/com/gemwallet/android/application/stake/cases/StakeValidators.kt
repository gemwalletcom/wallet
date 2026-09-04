package com.gemwallet.android.application.stake.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.DelegationValidator
import kotlinx.coroutines.flow.Flow

interface GetValidators {
    operator fun invoke(assetId: AssetId): Flow<List<DelegationValidator>>
}

