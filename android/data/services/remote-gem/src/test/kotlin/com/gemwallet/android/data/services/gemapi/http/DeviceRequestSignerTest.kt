package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import org.junit.Assert.assertEquals
import org.junit.Test

class DeviceRequestSignerTest {

    @Test
    fun `constructor does not read device key`() {
        val getDeviceId = RecordingGetDeviceId()

        GemDeviceRequestSigner(getDeviceId)

        assertEquals(0, getDeviceId.keyReadCount)
    }

    private class RecordingGetDeviceId : GetDeviceId {
        var keyReadCount = 0
            private set

        override suspend fun getDeviceId(): String = "public"

        override suspend fun getDeviceKey(): String {
            keyReadCount += 1
            return "private"
        }
    }
}
