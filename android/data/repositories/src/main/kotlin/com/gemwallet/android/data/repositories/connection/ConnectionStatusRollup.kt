package com.gemwallet.android.data.repositories.connection

import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionComponentHealth
import com.wallet.core.primitives.ConnectionStatus

internal val ConnectionComponent.failureStatus: ConnectionStatus
    get() = when (this) {
        ConnectionComponent.Internet -> ConnectionStatus.NoInternet
        ConnectionComponent.Api,
        ConnectionComponent.Nodes,
        ConnectionComponent.Stream -> ConnectionStatus.NoService
    }

internal val ConnectionStatus.severity: Int
    get() = when (this) {
        ConnectionStatus.Online -> 0
        ConnectionStatus.NoService -> 1
        ConnectionStatus.NoInternet -> 2
    }

internal fun Map<ConnectionComponent, ConnectionComponentHealth>.rollup(): ConnectionStatus = this
    .filterValues { !it.isHealthy }
    .keys
    .map { it.failureStatus }
    .maxByOrNull { it.severity } ?: ConnectionStatus.Online
