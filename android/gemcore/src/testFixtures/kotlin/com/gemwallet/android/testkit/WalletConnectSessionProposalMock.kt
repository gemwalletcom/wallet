package com.gemwallet.android.testkit

import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal

fun mockWalletConnectSessionProposal(): WalletConnectSessionProposal {
    val metadata = mockApplicationMetadata()
    return WalletConnectSessionProposal(
        name = metadata.name,
        description = metadata.description,
        url = metadata.url,
        icons = listOf(metadata.icon),
        requiredNamespaces = emptyMap(),
        optionalNamespaces = emptyMap(),
        proposerPublicKey = "key",
        pairingTopic = "pairing",
        properties = null,
    )
}
