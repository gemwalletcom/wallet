package com.gemwallet.android.data.service.walletconnect.reown.testkit

import com.reown.walletkit.client.Wallet

fun mockSessionProposal(
    chainId: String = "tron:728126428",
    optional: Boolean = false,
): Wallet.Model.SessionProposal {
    val namespaces = mapOf("tron" to Wallet.Model.Namespace.Proposal(chains = listOf(chainId), methods = listOf("tron_signMessage"), events = emptyList()))
    return Wallet.Model.SessionProposal(
        pairingTopic = "pairing",
        name = "TRON dapp",
        description = "",
        url = "https://example.com",
        icons = emptyList(),
        redirect = "",
        requiredNamespaces = if (optional) emptyMap() else namespaces,
        optionalNamespaces = if (optional) namespaces else emptyMap(),
        properties = null,
        proposerPublicKey = "proposer",
        relayProtocol = "irn",
        relayData = null,
        scopedProperties = null,
        requests = null,
    )
}
