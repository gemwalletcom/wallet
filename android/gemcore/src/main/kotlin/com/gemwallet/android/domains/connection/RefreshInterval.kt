package com.gemwallet.android.domains.connection

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.ConnectionStatus
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemRefreshKind
import java.time.Duration

private val connectionService: GemConnectionService by lazy { GemConnectionService() }

fun ConnectionStatus.refreshInterval(kind: GemRefreshKind): Duration = connectionService.refreshInterval(kind, toGem())
