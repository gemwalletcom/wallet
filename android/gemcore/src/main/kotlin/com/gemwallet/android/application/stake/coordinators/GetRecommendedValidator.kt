package com.gemwallet.android.application.stake.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.DelegationValidator
import kotlinx.coroutines.flow.Flow

interface GetRecommendedValidator {
    operator fun invoke(assetId: AssetId): Flow<DelegationValidator?>
}
