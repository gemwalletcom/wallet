package com.gemwallet.android.data.services.gemstone.connection

import com.wallet.core.primitives.ConnectionComponent
import kotlinx.coroutines.flow.Flow

interface ConnectionComponentMonitor {
    val component: ConnectionComponent
    fun healthFlow(): Flow<Boolean>
}
