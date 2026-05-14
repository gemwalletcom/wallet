package com.gemwallet.android.application.stake.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import kotlinx.coroutines.flow.Flow

interface GetDelegations {
    operator fun invoke(assetId: AssetId, owner: String): Flow<List<Delegation>>
}
