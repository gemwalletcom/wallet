package com.gemwallet.android.data.repositories.device

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemDeviceService

@OptIn(ExperimentalCoroutinesApi::class)
class DeviceObserverServiceTest {

    private val wallet = mockWallet(id = "wallet-1", accounts = listOf(mockAccount(chain = Chain.Ethereum)))
    private val wallets = MutableStateFlow(listOf(wallet))
    private val walletsRepository = mockk<WalletsRepository> {
        every { getAll() } returns wallets
    }
    private val deviceService = mockk<GemDeviceService>(relaxed = true)

    @Test
    fun synchronizesOnEveryWalletsChange() = runTest {
        val subject = service()
        subject.start()
        advanceUntilIdle()

        wallets.value = listOf(wallet.copy(accounts = wallet.accounts + mockAccount(chain = Chain.Bitcoin)))
        advanceUntilIdle()
        subject.stop()

        coVerify(exactly = 2) { deviceService.synchronizeIfNeeded() }
    }

    private fun TestScope.service() = DeviceObserverService(
        walletsRepository = walletsRepository,
        deviceService = deviceService,
        scope = this,
    )
}
