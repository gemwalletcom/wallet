package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.model.AmountParams

sealed interface AmountTitle {
    data object Send : AmountTitle
    data object Deposit : AmountTitle
    data object Withdraw : AmountTitle
    data class Stake(val action: AmountParams.Stake) : AmountTitle
    data class Perpetual(val action: PerpetualPositionAction) : AmountTitle
}
