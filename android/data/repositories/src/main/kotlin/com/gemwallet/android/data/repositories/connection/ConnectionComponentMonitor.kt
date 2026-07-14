package com.gemwallet.android.data.repositories.connection

import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionComponentHealth
import kotlinx.coroutines.flow.Flow

interface ConnectionComponentMonitor {
    val component: ConnectionComponent
    fun healthFlow(): Flow<ConnectionComponentHealth>
}
