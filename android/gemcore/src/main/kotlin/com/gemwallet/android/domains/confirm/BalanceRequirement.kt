package com.gemwallet.android.domains.confirm

import uniffi.gemstone.GemConfirmException
import java.math.BigInteger

data class BalanceRequirement(
    val required: BigInteger,
    val available: BigInteger,
) {
    val shortfall: BigInteger
        get() = (required - available).max(BigInteger.ZERO)
}

val GemConfirmException.balanceRequirement: BalanceRequirement?
    get() = when (this) {
        is GemConfirmException.InsufficientBalance -> BalanceRequirement(BigInteger(required), BigInteger(available))
        is GemConfirmException.MinimumAccountBalanceTooLow -> BalanceRequirement(BigInteger(required), BigInteger(available))
        is GemConfirmException.InsufficientNetworkFee -> if (required != null && available != null) BalanceRequirement(BigInteger(required), BigInteger(available)) else null
        else -> null
    }
