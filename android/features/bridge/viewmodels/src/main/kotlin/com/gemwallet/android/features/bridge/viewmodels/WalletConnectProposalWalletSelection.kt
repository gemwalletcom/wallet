package com.gemwallet.android.features.bridge.viewmodels

import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionProposal
import com.gemwallet.android.data.repositories.bridge.fromWalletConnectChainId
import com.gemwallet.android.data.repositories.bridge.walletConnectNamespace
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet as GemWallet
import com.wallet.core.primitives.WalletType

internal data class WalletConnectProposalChains(
    val required: Set<Chain>,
    val optional: Set<Chain>,
)

internal fun WalletConnectSessionProposal.supportedWalletConnectProposalChains(): WalletConnectProposalChains? {
    val requiredValues = requiredNamespaces.values.flatMap { it.chains.orEmpty() }
    val requiredChains = requiredValues.mapNotNull { Chain.fromWalletConnectChainId(it) }
    if (requiredChains.size != requiredValues.size) {
        return null
    }
    val optionalChains = optionalNamespaces.values
        .flatMap { it.chains.orEmpty() }
        .mapNotNull { Chain.fromWalletConnectChainId(it) }
        .toSet()

    return WalletConnectProposalChains(
        required = requiredChains.toSet(),
        optional = optionalChains,
    )
}

internal fun List<GemWallet>.walletsSupportingWalletConnectProposal(
    chains: WalletConnectProposalChains,
): List<GemWallet> {
    return filter { wallet ->
        wallet.type != WalletType.View && wallet.supports(chains)
    }.sortedBy { it.type }
}

private fun GemWallet.supports(chains: WalletConnectProposalChains): Boolean {
    val walletChains = accounts.map { it.chain }.filter { it.walletConnectNamespace() != null }.toSet()
    if (walletChains.isEmpty()) {
        return false
    }
    if (chains.required.isNotEmpty()) {
        return walletChains.containsAll(chains.required)
    }
    return chains.optional.isEmpty() || walletChains.any { it in chains.optional }
}
