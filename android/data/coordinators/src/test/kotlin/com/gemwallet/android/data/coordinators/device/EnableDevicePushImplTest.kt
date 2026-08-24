package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.cases.device.SwitchPushEnabled
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import org.junit.Test

class EnableDevicePushImplTest {

    @Test
    fun invoke_switchesPushEnabled() = runBlocking {
        val switchPushEnabled = mockk<SwitchPushEnabled> {
            coEvery { switchPushEnabled(any()) } returns Unit
        }

        EnableDevicePushImpl(switchPushEnabled).invoke()

        coVerify(exactly = 1) { switchPushEnabled.switchPushEnabled(true) }
    }
}
