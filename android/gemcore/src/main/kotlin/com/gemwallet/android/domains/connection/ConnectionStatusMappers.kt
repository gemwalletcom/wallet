package com.gemwallet.android.domains.connection

import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionStatus
import uniffi.gemstone.connectionStatus

fun List<ConnectionComponent>.toConnectionStatus(): ConnectionStatus =
    connectionStatus(map { it.toJson() }).decodeJson()
