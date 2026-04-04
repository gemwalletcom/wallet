package com.gemwallet.android.data.repositories.stream

import kotlin.math.exp
import kotlin.math.min

class ExponentialReconnection(
    private val multiplier: Double = 0.3,
    private val maxDelay: Double = 60.0,
) {
    fun reconnectAfterMs(attempt: Int): Long {
        val seconds = min(multiplier * exp(attempt.toDouble()), maxDelay)
        return (seconds * 1000).toLong()
    }
}
