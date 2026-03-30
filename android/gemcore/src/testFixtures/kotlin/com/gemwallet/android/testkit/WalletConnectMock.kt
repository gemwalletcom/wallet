package com.gemwallet.android.testkit

import com.reown.android.Core
import com.reown.walletkit.client.Wallet
import java.net.URI

fun mockWalletConnectProposal(
    pairingTopic: String = "pairing-topic",
    name: String = "Proposal App",
    description: String = "Proposal description",
    url: String = "https://proposal.example",
    icons: List<URI> = listOf(URI.create("https://proposal.example/icon.png")),
    redirect: String = "proposal://redirect",
    proposerPublicKey: String = "public-key",
) = Wallet.Model.SessionProposal(
    pairingTopic = pairingTopic,
    name = name,
    description = description,
    url = url,
    icons = icons,
    redirect = redirect,
    requiredNamespaces = emptyMap(),
    optionalNamespaces = emptyMap(),
    properties = emptyMap(),
    proposerPublicKey = proposerPublicKey,
    relayProtocol = "irn",
    relayData = "",
    scopedProperties = emptyMap(),
    requests = Wallet.Model.ProposalRequests(emptyList()),
)

fun mockWalletConnectSession(
    pairingTopic: String = "pairing-topic",
    topic: String = "settled-topic",
    expiry: Long = 1_900_000_000L,
    name: String = "Settled App",
    description: String = "Settled description",
    url: String = "https://settled.example",
    icon: String = "https://settled.example/icon.png",
    redirect: String = "settled://redirect",
) = Wallet.Model.Session(
    pairingTopic = pairingTopic,
    topic = topic,
    expiry = expiry,
    requiredNamespaces = emptyMap(),
    optionalNamespaces = emptyMap(),
    namespaces = mapOf(
        "eip155" to Wallet.Model.Namespace.Session(
            chains = listOf("eip155:1"),
            accounts = listOf("eip155:1:0xabc"),
            methods = emptyList(),
            events = emptyList(),
        )
    ),
    metaData = Core.Model.AppMetaData(
        name = name,
        description = description,
        url = url,
        icons = listOf(icon),
        redirect = redirect,
        appLink = "",
        linkMode = false,
        verifyUrl = "",
    ),
)
