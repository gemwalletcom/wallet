package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.Wallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Test

class EnableDevicePushImplTest {

    @Test
    fun invoke_switchesPushEnabledWithCurrentWallets() = runBlocking {
        val wallets = listOf(mockk<Wallet>())
        val walletsRepository = mockk<WalletsRepository> {
            coEvery { getAll() } returns flowOf(wallets)
        }
        val switchPushEnabled = mockk<SwitchPushEnabled> {
            coEvery { switchPushEnabled(any(), any()) } returns Unit
        }

        EnableDevicePushImpl(switchPushEnabled, walletsRepository).invoke()

        coVerify(exactly = 1) { switchPushEnabled.switchPushEnabled(true, wallets) }
    }

    @Test
    fun invoke_withNoWallets_switchesPushEnabledWithEmptyList() = runBlocking {
        val walletsRepository = mockk<WalletsRepository> {
            coEvery { getAll() } returns flowOf(emptyList())
        }
        val switchPushEnabled = mockk<SwitchPushEnabled> {
            coEvery { switchPushEnabled(any(), any()) } returns Unit
        }

        EnableDevicePushImpl(switchPushEnabled, walletsRepository).invoke()

        coVerify(exactly = 1) { switchPushEnabled.switchPushEnabled(true, emptyList()) }
    }
}
