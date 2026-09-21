package com.gemwallet.android.testkit

import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnectionSession
import com.wallet.core.primitives.WalletConnectionState

fun mockWalletConnectionSession(id: String = "session-1", sessionId: String = "topic", expireAt: Long = 0, metadata: ApplicationMetadata = mockApplicationMetadata()) = WalletConnectionSession(
    id = id,
    sessionId = sessionId,
    state = WalletConnectionState.Active,
    chains = listOf(Chain.Ethereum),
    createdAt = 0,
    expireAt = expireAt,
    metadata = metadata,
)
