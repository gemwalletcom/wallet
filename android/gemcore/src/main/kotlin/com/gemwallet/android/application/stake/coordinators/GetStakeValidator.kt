package com.gemwallet.android.application.stake.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.DelegationValidator

interface GetStakeValidator {
    suspend operator fun invoke(assetId: AssetId, validatorId: String): DelegationValidator?
}
