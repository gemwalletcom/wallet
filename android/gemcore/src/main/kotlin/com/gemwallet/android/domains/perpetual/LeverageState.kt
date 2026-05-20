package com.gemwallet.android.domains.perpetual

import com.wallet.core.primitives.PerpetualDirection

data class LeverageState(
    val current: Int,
    val options: List<Int>,
    val direction: PerpetualDirection,
)
