package com.gemwallet.android.ext

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.onEach

fun tickerFlow(delay: Long, onTick: (Long) -> Unit): Flow<Long> {
    require(delay > 0) { "delay must be greater than 0" }
    return flow {
        while (true) {
            delay(delay)
            emit(System.currentTimeMillis())
        }
    }.onEach(onTick)
}
