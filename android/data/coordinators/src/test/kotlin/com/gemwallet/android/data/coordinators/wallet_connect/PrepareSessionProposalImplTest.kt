package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectionSessionProposal
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemSessionProposal
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus

class PrepareSessionProposalImplTest {

    private val currentWallet = mockWallet(id = "wallet-2")
    private val wallets = listOf(mockWallet(id = "wallet-1"), currentWallet)
    private val metadata = mockApplicationMetadata()
    private val walletConnectService = mockk<GemWalletConnectServiceInterface> {
        every { applicationMetadata(metadata.name, metadata.description, metadata.url, listOf(metadata.icon)) } returns metadata.toGem()
    }
    private val subject = PrepareSessionProposalImpl(walletConnectService)

    @Test
    fun prepareSessionProposal_mapsCoreProposal() = runTest {
        val proposal = mockWalletConnectionSessionProposal(defaultWallet = currentWallet, wallets = wallets)
        coEvery {
            walletConnectService.prepareSessionProposal(
                requiredChainIds = listOf("eip155:1"),
                optionalChainIds = emptyList(),
                metadata = metadata.toGem(),
                origin = "https://app.example",
                validation = WalletConnectionVerificationStatus.VERIFIED,
            )
        } returns GemSessionProposal(proposal.toGem(), WalletConnectionVerificationStatus.VERIFIED)

        val prepared = prepare(requiredChainIds = listOf("eip155:1"))

        assertEquals(proposal, prepared?.proposal)
        assertEquals(WalletConnectionVerificationStatus.VERIFIED, prepared?.verificationStatus)
    }

    @Test
    fun prepareSessionProposal_failsWhenCoreRejects() = runTest {
        coEvery { walletConnectService.prepareSessionProposal(any(), any(), any(), any(), any()) } throws GemWalletConnectException.UnsupportedWallets()

        assertTrue(runCatching { prepare(requiredChainIds = listOf("eip155:1", "cosmos:unknown-9")) }.isFailure)
    }

    private suspend fun prepare(requiredChainIds: List<String>) = subject(
        name = metadata.name,
        description = metadata.description,
        url = metadata.url,
        icons = listOf(metadata.icon),
        requiredChainIds = requiredChainIds,
        optionalChainIds = emptyList(),
        origin = "https://app.example",
        validation = WalletConnectionVerificationStatus.VERIFIED,
    )
}
