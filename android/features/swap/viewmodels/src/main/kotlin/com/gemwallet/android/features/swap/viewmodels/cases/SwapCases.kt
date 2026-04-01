package com.gemwallet.android.features.swap.viewmodels.cases

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flow
import java.math.BigDecimal

internal fun tickerFlow(value: BigDecimal): Flow<Long> {
    return if (value == BigDecimal.ZERO) {
        emptyFlow()
    } else {
        flow {
            while (true) {
                delay(30 * 1000)
                emit(System.currentTimeMillis())
            }
        }
    }
}
