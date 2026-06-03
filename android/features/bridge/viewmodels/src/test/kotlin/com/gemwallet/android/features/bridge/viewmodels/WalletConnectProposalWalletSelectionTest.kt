package com.gemwallet.android.features.bridge.viewmodels

import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletConnectProposalWalletSelectionTest {

    @Test
    fun requiredChainsNeedWalletSuperset() {
        val ethereumWallet = mockWallet(id = "ethereum", accounts = listOf(mockAccount(chain = Chain.Ethereum)))
        val ethereumPolygonWallet = mockWallet(
            id = "ethereum-polygon",
            accounts = listOf(mockAccount(chain = Chain.Ethereum), mockAccount(chain = Chain.Polygon)),
        )

        val wallets = listOf(ethereumWallet, ethereumPolygonWallet).walletsSupportingWalletConnectProposal(
            chains = WalletConnectProposalChains(
                required = setOf(Chain.Ethereum, Chain.Polygon),
                optional = emptySet(),
            ),
        )

        assertEquals(listOf(ethereumPolygonWallet.id), wallets.map { it.id })
    }

    @Test
    fun optionalChainsMatchAnySupportedWalletChain() {
        val solanaWallet = mockWallet(id = "solana", accounts = listOf(mockAccount(chain = Chain.Solana)))
        val ethereumWallet = mockWallet(id = "ethereum", accounts = listOf(mockAccount(chain = Chain.Ethereum)))
        val bitcoinWallet = mockWallet(id = "bitcoin", accounts = listOf(mockAccount(chain = Chain.Bitcoin)))

        val wallets = listOf(solanaWallet, ethereumWallet, bitcoinWallet).walletsSupportingWalletConnectProposal(
            chains = WalletConnectProposalChains(
                required = emptySet(),
                optional = setOf(Chain.Solana, Chain.Ethereum),
            ),
        )

        assertEquals(listOf(solanaWallet.id, ethereumWallet.id), wallets.map { it.id })
    }

    @Test
    fun emptyProposalChainsStillRequireWalletConnectSupportedWalletChain() {
        val ethereumWallet = mockWallet(id = "ethereum", accounts = listOf(mockAccount(chain = Chain.Ethereum)))
        val bitcoinWallet = mockWallet(id = "bitcoin", accounts = listOf(mockAccount(chain = Chain.Bitcoin)))

        val wallets = listOf(ethereumWallet, bitcoinWallet).walletsSupportingWalletConnectProposal(
            chains = WalletConnectProposalChains(required = emptySet(), optional = emptySet()),
        )

        assertEquals(listOf(ethereumWallet.id), wallets.map { it.id })
    }

    @Test
    fun viewWalletsAreNotEligible() {
        val regularWallet = mockWallet(id = "regular", accounts = listOf(mockAccount(chain = Chain.Ethereum)))
        val viewWallet = mockWallet(
            id = "view",
            type = WalletType.View,
            accounts = listOf(mockAccount(chain = Chain.Ethereum)),
        )

        val wallets = listOf(viewWallet, regularWallet).walletsSupportingWalletConnectProposal(
            chains = WalletConnectProposalChains(required = setOf(Chain.Ethereum), optional = emptySet()),
        )

        assertEquals(listOf(regularWallet.id), wallets.map { it.id })
    }

    @Test
    fun optionalChainsIgnoredWhenRequiredChainsExist() {
        val ethereumWallet = mockWallet(id = "ethereum", accounts = listOf(mockAccount(chain = Chain.Ethereum)))
        val solanaWallet = mockWallet(id = "solana", accounts = listOf(mockAccount(chain = Chain.Solana)))
        val ethereumSolanaWallet = mockWallet(
            id = "ethereum-solana",
            accounts = listOf(mockAccount(chain = Chain.Ethereum), mockAccount(chain = Chain.Solana)),
        )

        val wallets = listOf(ethereumWallet, solanaWallet, ethereumSolanaWallet).walletsSupportingWalletConnectProposal(
            chains = WalletConnectProposalChains(
                required = setOf(Chain.Ethereum, Chain.Solana),
                optional = setOf(Chain.Polygon),
            ),
        )

        assertEquals(listOf(ethereumSolanaWallet.id), wallets.map { it.id })
    }
}
