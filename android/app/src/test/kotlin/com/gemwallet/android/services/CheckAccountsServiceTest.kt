package com.gemwallet.android.services

import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import org.junit.Test
import uniffi.gemstone.GemAppStartService

class CheckAccountsServiceTest {

    private val appStartService = mockk<GemAppStartService> {
        coEvery { setupWallets() } returns emptyList()
    }

    @Test
    fun invoke_setsUpWalletsThroughCore() = runBlocking {
        CheckAccountsService(appStartService).invoke()

        coVerify(exactly = 1) { appStartService.setupWallets() }
    }
}
