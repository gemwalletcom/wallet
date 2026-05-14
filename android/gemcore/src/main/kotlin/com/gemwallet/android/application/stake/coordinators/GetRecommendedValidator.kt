package com.gemwallet.android.application.stake.coordinators

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationValidator
import kotlinx.coroutines.flow.Flow

interface GetRecommendedValidator {
    operator fun invoke(chain: Chain): Flow<DelegationValidator?>
}
