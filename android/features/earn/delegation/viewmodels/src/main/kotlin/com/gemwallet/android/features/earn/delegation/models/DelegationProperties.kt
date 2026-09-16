package com.gemwallet.android.features.earn.delegation.models

import com.gemwallet.android.ui.models.RewardsInfoUIModel
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.GemDelegationRow
import uniffi.gemstone.GemDelegationStatus

class DelegationProperties(
    val rows: List<GemDelegationRow>,
    val validator: DelegationValidator,
    val validatorName: String,
    val validatorUrl: String?,
    val status: GemDelegationStatus,
    val availableIn: String,
    val rewards: RewardsInfoUIModel,
)
