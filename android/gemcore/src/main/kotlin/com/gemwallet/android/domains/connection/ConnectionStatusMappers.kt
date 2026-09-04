package com.gemwallet.android.domains.connection

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionStatus
import uniffi.gemstone.connectionStatus

fun List<ConnectionComponent>.toConnectionStatus(): ConnectionStatus =
    connectionStatus(map { it.toGem() }).toPrimitives()
