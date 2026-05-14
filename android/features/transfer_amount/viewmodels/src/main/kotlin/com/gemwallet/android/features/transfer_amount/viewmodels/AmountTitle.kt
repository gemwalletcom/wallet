package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.PerpetualDirection

sealed interface AmountTitle {
    data object Send : AmountTitle
    data class Stake(val action: AmountParams.Stake) : AmountTitle
    data class Perpetual(val direction: PerpetualDirection) : AmountTitle
}
