package com.gemwallet.android.domains.confirm

import java.math.BigInteger

data class BalanceRequirement(
    val required: BigInteger,
    val available: BigInteger,
) {
    val shortfall: BigInteger
        get() = (required - available).max(BigInteger.ZERO)
}
