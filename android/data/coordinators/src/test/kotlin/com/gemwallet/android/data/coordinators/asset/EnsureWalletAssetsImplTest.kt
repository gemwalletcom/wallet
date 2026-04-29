package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test

class EnsureWalletAssetsImplTest {

    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val enableAsset = mockk<EnableAsset>(relaxed = true)

    private val subject = EnsureWalletAssetsImpl(
        assetsRepository = assetsRepository,
        enableAsset = enableAsset,
    )

    @Test
    fun ensureWalletAssets_enablesOnlyMissingWalletAssets() = runTest {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val ethereumAccount = mockAccount(chain = Chain.Ethereum, address = "0x-current")
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(ethereumAccount),
        )

        coEvery {
            assetsRepository.hasWalletAssets("wallet-1", listOf(bitcoin.id, ethereum.id))
        } returns setOf(bitcoin.id)

        subject.ensureWalletAssets(wallet, listOf(bitcoin.id, ethereum.id, ethereum.id))

        coVerify(exactly = 1) { enableAsset("wallet-1", listOf(ethereum.id)) }
    }

    @Test
    fun ensureWalletAssets_skipsExistingWalletAssets() = runTest {
        val bitcoin = mockAsset()
        val bitcoinAccount = mockAccount(chain = Chain.Bitcoin, address = "bc1-current")
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(bitcoinAccount),
        )

        coEvery { assetsRepository.hasWalletAssets("wallet-1", listOf(bitcoin.id)) } returns setOf(bitcoin.id)

        subject.ensureWalletAssets(wallet, listOf(bitcoin.id))

        coVerify(exactly = 0) { enableAsset(any(), any<List<com.wallet.core.primitives.AssetId>>()) }
        coVerify(exactly = 0) { enableAsset(any(), any<com.wallet.core.primitives.AssetId>()) }
    }

    @Test
    fun ensureWalletAssets_skipsAssetsWithoutAccount() = runTest {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Bitcoin)),
        )

        coEvery {
            assetsRepository.hasWalletAssets("wallet-1", listOf(bitcoin.id, ethereum.id))
        } returns emptySet()

        subject.ensureWalletAssets(wallet, listOf(bitcoin.id, ethereum.id))

        coVerify(exactly = 1) { enableAsset("wallet-1", listOf(bitcoin.id)) }
    }
}
