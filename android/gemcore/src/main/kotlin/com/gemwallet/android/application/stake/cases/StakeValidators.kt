package com.gemwallet.android.application.stake.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import kotlinx.coroutines.flow.Flow

interface GetValidators {
    operator fun invoke(assetId: AssetId, providerType: StakeProviderType = StakeProviderType.Stake): Flow<List<DelegationValidator>>
}

