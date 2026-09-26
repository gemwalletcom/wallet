package com.gemwallet.android.application.connection.cases

import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemRefreshKind

interface ObserveRefreshInterval {
    fun refreshIntervalMillis(kind: GemRefreshKind): Flow<Long>
}
