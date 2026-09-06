package com.gemwallet.android.domains.stake

import com.wallet.core.primitives.Delegation
import java.math.BigInteger

fun Delegation.hasRewards(): Boolean = base.rewards > BigInteger.ZERO
