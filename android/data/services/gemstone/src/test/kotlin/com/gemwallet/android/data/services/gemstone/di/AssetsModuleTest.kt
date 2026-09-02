package com.gemwallet.android.data.services.gemstone.di

import io.mockk.mockk
import io.mockk.verify
import okhttp3.OkHttpClient
import org.junit.Test
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemDeviceKeyService

class AssetsModuleTest {

    @Test
    fun `stream connection construction does not read device key`() {
        val deviceKeyService = mockk<GemDeviceKeyService>(relaxed = true)

        AssetsModule.provideStreamConnection(
            deviceKeyService = deviceKeyService,
            okHttpClient = OkHttpClient(),
            connectionService = mockk<GemConnectionService>(relaxed = true),
        )

        verify(exactly = 0) { deviceKeyService.keyPair() }
    }
}
