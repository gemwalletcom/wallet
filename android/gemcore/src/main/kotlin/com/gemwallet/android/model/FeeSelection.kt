package com.gemwallet.android.model

import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

sealed interface FeeSelection {
    data class Preset(val priority: FeePriority) : FeeSelection
    data class Custom(val gasPrice: BigInteger) : FeeSelection
}
