package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.Chain
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test

class PrefetchAssetsImplTest {

    private val gemApiClient = mockk<GemApiClient>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)

    private val subject = PrefetchAssetsImpl(
        gemApiClient = gemApiClient,
        assetsRepository = assetsRepository,
    )

    @Test
    fun prefetchAssets_loadsOnlyMissingWalletAssets() = runTest {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(
                mockAccount(chain = Chain.Bitcoin, address = "bc1-address"),
                mockAccount(chain = Chain.Ethereum, address = "0x-address"),
            ),
        )
        val ethereumBasic = mockAssetBasic(asset = ethereum, rank = 42)

        coEvery { assetsRepository.hasAsset(wallet.id, bitcoin.id) } returns true
        coEvery { assetsRepository.hasAsset(wallet.id, ethereum.id) } returns false
        coEvery { gemApiClient.getAssets(listOf(ethereum.id)) } returns listOf(ethereumBasic)

        subject.prefetchAssets(wallet, listOf(bitcoin.id, ethereum.id, ethereum.id))

        coVerify { gemApiClient.getAssets(listOf(ethereum.id)) }
        coVerify(exactly = 0) {
            assetsRepository.add(
                walletId = wallet.id,
                accountAddress = any(),
                asset = match<AssetBasic> { it.asset.id == bitcoin.id },
                visible = true,
            )
        }
        coVerify {
            assetsRepository.add(
                walletId = wallet.id,
                accountAddress = "0x-address",
                asset = ethereumBasic,
                visible = true,
            )
        }
        coVerify { assetsRepository.updateBalances(ethereum.id) }
    }

    @Test
    fun prefetchAssets_skipsLoadWhenAllAssetsAlreadyExist() = runTest {
        val bitcoin = mockAsset()
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(
                mockAccount(chain = Chain.Bitcoin, address = "bc1-address"),
            ),
        )

        coEvery { assetsRepository.hasAsset(wallet.id, bitcoin.id) } returns true

        subject.prefetchAssets(wallet, listOf(bitcoin.id))

        coVerify(exactly = 0) { gemApiClient.getAssets(any()) }
        coVerify(exactly = 0) { assetsRepository.add(any(), any(), any<AssetBasic>(), any()) }
        coVerify(exactly = 0) { assetsRepository.updateBalances(any()) }
    }
}
