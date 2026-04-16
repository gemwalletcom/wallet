package com.gemwallet.android.domains.stake

import com.wallet.core.primitives.Delegation
import java.math.BigInteger

fun Delegation.rewardsBalance(): BigInteger = base.rewards.toBigIntegerOrNull() ?: BigInteger.ZERO

fun Delegation.hasRewards(): Boolean = rewardsBalance() > BigInteger.ZERO

fun Iterable<Delegation>.sumRewardsBalance(): BigInteger {
    return fold(BigInteger.ZERO) { total, delegation -> total + delegation.rewardsBalance() }
}
