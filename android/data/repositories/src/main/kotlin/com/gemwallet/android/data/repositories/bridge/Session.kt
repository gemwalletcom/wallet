package com.gemwallet.android.data.repositories.bridge

import com.gemwallet.android.data.service.store.database.entities.DbConnection
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toChainType
import com.gemwallet.android.ext.walletConnectAppName
import com.gemwallet.android.ext.walletConnectIcon
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.WalletConnectionState
import com.wallet.core.primitives.Wallet as GemWallet
import uniffi.gemstone.ChainAddress as WalletConnectChainAddress
import uniffi.gemstone.WalletConnect

internal fun WalletConnectSession.toConnectionRecord(
    walletId: String,
    createdAt: Long,
): DbConnection {
    return DbConnection(
        id = topic,
        walletId = walletId,
        sessionId = topic,
        state = WalletConnectionState.Active,
        chains = accounts().map { it.chain }.distinct(),
        createdAt = createdAt,
        expireAt = expiry.secondsToMillis(),
        appName = walletConnectAppName(metadata?.name, metadata?.url),
        appDescription = metadata?.description ?: "",
        appUrl = metadata?.url ?: "",
        appIcon = listOf(metadata?.icon.orEmpty()).walletConnectIcon(),
        redirectNative = redirect,
        redirectUniversal = redirect,
    )
}

internal fun WalletConnectSession.accounts(): List<ChainAddress> {
    val walletConnect = WalletConnect()
    return namespaces.values
        .flatMap { it.accounts }
        .mapNotNull { walletConnect.parseAccount(it)?.toPrimitives() }
}

private fun WalletConnectChainAddress.toPrimitives(): ChainAddress? {
    val chain = chain.toChain() ?: return null
    return ChainAddress(chain, address)
}

internal fun WalletConnectSession.belongsTo(wallet: GemWallet): Boolean {
    return accounts().belongsTo(wallet)
}

internal fun List<ChainAddress>.belongsTo(wallet: GemWallet): Boolean {
    return any { sessionAccount ->
        wallet.getAccount(sessionAccount.chain)?.addressMatches(sessionAccount) == true
    }
}

private fun Account.addressMatches(sessionAccount: ChainAddress): Boolean {
    return when (chain.toChainType()) {
        ChainType.Ethereum -> address.equals(sessionAccount.address, ignoreCase = true)
        else -> address == sessionAccount.address
    }
}
