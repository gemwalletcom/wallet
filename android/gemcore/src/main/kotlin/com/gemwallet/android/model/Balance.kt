package com.gemwallet.android.model

import java.math.BigInteger

data class Balance(
    val available: BigInteger,
    val frozen: BigInteger,
    val locked: BigInteger,
    val staked: BigInteger,
    val pending: BigInteger,
    val rewards: BigInteger,
    val reserved: BigInteger,
    val withdrawable: BigInteger,
    val pendingUnconfirmed: BigInteger,
    val earn: BigInteger,
) {
    companion object {
        fun zero(): Balance = Balance(
            available = BigInteger.ZERO,
            frozen = BigInteger.ZERO,
            locked = BigInteger.ZERO,
            staked = BigInteger.ZERO,
            pending = BigInteger.ZERO,
            rewards = BigInteger.ZERO,
            reserved = BigInteger.ZERO,
            withdrawable = BigInteger.ZERO,
            pendingUnconfirmed = BigInteger.ZERO,
            earn = BigInteger.ZERO,
        )
    }
}
