package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Test

class EnsureWalletAssetsImplTest {

    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)

    private val subject = EnsureWalletAssetsImpl(
        assetsRepository = assetsRepository,
    )

    @Test
    fun ensureWalletAssets_addsOnlyMissingWalletAssets() = runTest {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val ethereumAccount = mockAccount(chain = Chain.Ethereum, address = "0x-current")
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(ethereumAccount),
        )
        val assetIds = listOf(bitcoin.id, ethereum.id)

        coEvery { assetsRepository.hasWalletAssets("wallet-1", assetIds) } returns setOf(bitcoin.id)
        every {
            assetsRepository.getTokensInfo(listOf(ethereum.id.toIdentifier()))
        } returns flowOf(listOf(mockAssetInfo(asset = ethereum)))

        subject.ensureWalletAssets(wallet, listOf(bitcoin.id, ethereum.id, ethereum.id))

        coVerify {
            assetsRepository.add(
                walletId = "wallet-1",
                asset = ethereum,
                visible = true,
            )
        }
        coVerify(exactly = 0) {
            assetsRepository.add(
                walletId = "wallet-1",
                asset = bitcoin,
                visible = true,
            )
        }
        coVerify { assetsRepository.updateBalances(ethereum.id) }
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

        coVerify(exactly = 0) { assetsRepository.add(any(), any<com.wallet.core.primitives.Asset>(), any()) }
        coVerify(exactly = 0) { assetsRepository.updateBalances(any()) }
    }
}
