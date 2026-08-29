package com.gemwallet.android.application.bridge.cases

import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.flow.Flow

interface GetWalletConnections {
    fun observeConnections(): Flow<List<WalletConnection>>

    fun observeConnection(connectionId: String): Flow<WalletConnection?>

    suspend fun getConnectionByTopic(topic: String): WalletConnection?
}
