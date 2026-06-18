package com.gemwallet.android.ext

import com.wallet.core.primitives.PerpetualBalance

val PerpetualBalance.total: Double
    get() = available + reserved
