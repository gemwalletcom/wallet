package com.gemwallet.android.domains.confirm

import uniffi.gemstone.GemBalanceRequirement
import java.math.BigInteger

data class BalanceRequirement(
    val required: BigInteger,
    val available: BigInteger,
    val shortfall: BigInteger,
)

fun GemBalanceRequirement.toPrimitives(): BalanceRequirement =
    BalanceRequirement(BigInteger(required), BigInteger(available), BigInteger(shortfall))
