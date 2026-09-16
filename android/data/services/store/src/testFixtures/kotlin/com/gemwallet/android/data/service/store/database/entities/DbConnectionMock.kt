package com.gemwallet.android.data.service.store.database.entities

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnectionState

fun mockDbConnection(
    id: String = "connection-1",
    walletId: String = "wallet-1",
) = DbConnection(
    id = id,
    walletId = walletId,
    sessionId = id,
    state = WalletConnectionState.Active,
    chains = listOf(Chain.Ethereum),
    createdAt = 1_000,
    expireAt = 2_000,
    appName = "App",
    appDescription = "Description",
    appUrl = "https://example.com",
    appIcon = "https://example.com/icon.png",
    redirectNative = null,
    redirectUniversal = null,
)
