package com.gemwallet.android.data.adapters.connection

import com.gemwallet.android.domains.connection.toConnectionStatus
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionStatus

internal val Map<ConnectionComponent, Boolean>.connectionStatus: ConnectionStatus
    get() = filterValues { !it }.keys.toList().toConnectionStatus()
