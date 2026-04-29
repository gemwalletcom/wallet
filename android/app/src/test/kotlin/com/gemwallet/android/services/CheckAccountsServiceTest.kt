package com.gemwallet.android.services

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.CreateAccountOperator
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.available
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coJustRun
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.Job
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Test

class CheckAccountsServiceTest {
    private val walletsRepository = mockk<WalletsRepository>(relaxed = true)
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val loadPrivateDataOperator = mockk<LoadPrivateDataOperator>(relaxed = true)
    private val passwordStore = mockk<PasswordStore>(relaxed = true)
    private val createAccountOperator = mockk<CreateAccountOperator>(relaxed = true)
    private val syncSubscription = mockk<SyncSubscription>(relaxed = true)

    private val subject = CheckAccountsService(
        walletsRepository = walletsRepository,
        assetsRepository = assetsRepository,
        loadPrivateDataOperator = loadPrivateDataOperator,
        passwordStore = passwordStore,
        createAccountOperator = createAccountOperator,
        syncSubscription = syncSubscription,
    )

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun invoke_repairsMissingNativeAssetsWithoutCreatingNewAccounts() = runBlocking {
        val accounts = Chain.available().map { chain ->
            mockAccount(chain = chain)
        }
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = accounts,
        )
        val nativeAssets = Chain.available()
            .filterNot { it == Chain.Ethereum }
            .map { chain -> mockAsset(chain = chain) }

        every { walletsRepository.getAll() } returns flowOf(listOf(wallet))
        coJustRun { assetsRepository.updateNativeAssetRanks() }
        every { assetsRepository.invalidateDefault(wallet) } returns Job()
        coJustRun { syncSubscription.syncSubscription(any()) }
        coJustRun { walletsRepository.updateWallet(any()) }
        coJustRun { walletsRepository.updateAccounts(any()) }
        coEvery { assetsRepository.getNativeAssets(wallet) } returns nativeAssets

        subject()

        coVerify(exactly = 1) { assetsRepository.updateNativeAssetRanks() }
        verify(exactly = 1) { walletsRepository.getAll() }
        coVerify(exactly = 1) { assetsRepository.getNativeAssets(wallet) }
        verify(exactly = 1) { assetsRepository.invalidateDefault(wallet) }
        verify(exactly = 0) { passwordStore.getPassword(any()) }
        verify(exactly = 0) { createAccountOperator(any(), any(), any()) }
        coVerify(exactly = 0) { loadPrivateDataOperator(any(), any()) }
        coVerify(exactly = 0) { walletsRepository.updateWallet(any()) }
        coVerify(exactly = 0) { walletsRepository.updateAccounts(any()) }
        coVerify(exactly = 0) { syncSubscription.syncSubscription(any()) }
    }

    @Test
    fun invoke_doesNotRepairWhenExpectedNativeAssetsExist() = runBlocking {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.available() } returns setOf(Chain.Solana)

        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Solana)),
        )
        val nativeAssets = listOf(
            mockAsset(chain = Chain.Solana),
            mockAsset(chain = Chain.Ethereum),
        )

        every { walletsRepository.getAll() } returns flowOf(listOf(wallet))
        coJustRun { assetsRepository.updateNativeAssetRanks() }
        coEvery { assetsRepository.getNativeAssets(wallet) } returns nativeAssets

        subject()

        coVerify(exactly = 1) { assetsRepository.getNativeAssets(wallet) }
        verify(exactly = 0) { assetsRepository.invalidateDefault(any()) }
        verify(exactly = 0) { passwordStore.getPassword(any()) }
        verify(exactly = 0) { createAccountOperator(any(), any(), any()) }
        coVerify(exactly = 0) { loadPrivateDataOperator(any(), any()) }
        coVerify(exactly = 0) { walletsRepository.updateWallet(any()) }
        coVerify(exactly = 0) { walletsRepository.updateAccounts(any()) }
        coVerify(exactly = 0) { syncSubscription.syncSubscription(any()) }
    }
}
