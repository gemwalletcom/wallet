package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.WalletConnectionSession
import uniffi.gemstone.GemSessionApproval
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemChainServiceInterface
import com.gemwallet.android.ext.toGem

fun WalletConnectSession.toConnectionSession(service: GemWalletConnectServiceInterface): WalletConnectionSession? {
    val metadata = metadata ?: return null
    return runCatching {
        service.session(
            topic = topic,
            accounts = namespaces.values.flatMap { it.accounts },
            expireAt = expiry,
            metadata = metadata.toGem(),
        )
    }.getOrNull()?.toPrimitives()
}

fun GemSessionApproval.toSupportedNamespaces(chainService: GemChainServiceInterface): Map<String, WalletConnectSessionNamespace> {
    return accounts
        .mapNotNull { account ->
            val namespace = chainService.caip2Namespace(account.chain) ?: return@mapNotNull null
            val reference = chainService.caip2Reference(account.chain) ?: return@mapNotNull null
            ApprovedAccount(namespace = namespace, chainId = "$namespace:$reference", address = account.address)
        }
        .groupBy { it.namespace }
        .mapValues { (_, approved) ->
            WalletConnectSessionNamespace(
                chains = approved.map { it.chainId }.distinct(),
                methods = methods,
                events = events,
                accounts = approved.map { it.accountId },
            )
        }
}

private data class ApprovedAccount(
    val namespace: String,
    val chainId: String,
    val address: String,
) {
    val accountId: String get() = "$chainId:$address"
}
