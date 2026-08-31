package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.WalletConnectionSessionProposal
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
    private val metadata = ApplicationMetadata(
        name = "App",
        description = "Description",
        url = "https://app.example",
        icon = "https://app.example/icon.png",
        source = ApplicationMetadataSource.WalletConnect,
    )
    private val walletConnectService = mockk<GemWalletConnectServiceInterface> {
        every { applicationMetadata(metadata.name, metadata.description, metadata.url, listOf(metadata.icon)) } returns metadata.toJson()
    }
    private val subject = PrepareSessionProposalImpl(walletConnectService)

    @Test
    fun prepareSessionProposal_mapsCoreProposal() = runTest {
        val proposal = WalletConnectionSessionProposal(defaultWallet = currentWallet, wallets = wallets, metadata = metadata)
        every {
            walletConnectService.prepareSessionProposal(
                requiredChainIds = listOf("eip155:1"),
                optionalChainIds = emptyList(),
                metadata = metadata.toJson(),
                origin = "https://app.example",
                validation = WalletConnectionVerificationStatus.VERIFIED,
            )
        } returns GemSessionProposal(proposal.toJson(), WalletConnectionVerificationStatus.VERIFIED)

        val prepared = prepare(requiredChainIds = listOf("eip155:1"))

        assertEquals(proposal, prepared?.proposal)
        assertEquals(WalletConnectionVerificationStatus.VERIFIED, prepared?.verificationStatus)
    }

    @Test
    fun prepareSessionProposal_failsWhenCoreRejects() = runTest {
        every { walletConnectService.prepareSessionProposal(any(), any(), any(), any(), any()) } throws GemWalletConnectException.UnsupportedWallets()

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
