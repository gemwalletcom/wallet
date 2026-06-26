package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.application.SecurityStore
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class GetDeviceIdImplTest {

    @Test
    fun `constructor does not read secure storage`() {
        val store = RecordingSecurityStore(
            values = mapOf(
                GetDeviceIdImpl.Keys.PrivateKey.toString() to "private",
                GetDeviceIdImpl.Keys.PublicKey.toString() to "public",
            )
        )

        GetDeviceIdImpl(store)

        assertEquals(0, store.readCount)
    }

    @Test
    fun `device keys are loaded lazily and cached`() = runTest {
        val store = RecordingSecurityStore(
            values = mapOf(
                GetDeviceIdImpl.Keys.PrivateKey.toString() to "private",
                GetDeviceIdImpl.Keys.PublicKey.toString() to "public",
            )
        )
        val deviceId = GetDeviceIdImpl(store)

        assertEquals("public", deviceId.getDeviceId())
        assertEquals("private", deviceId.getDeviceKey())

        assertEquals(2, store.readCount)
    }

    private class RecordingSecurityStore(
        private val values: Map<String, String>,
    ) : SecurityStore<Any> {

        var readCount = 0
            private set

        override suspend fun getValue(key: Any): String {
            readCount += 1
            return values.getValue(key.toString())
        }

        override suspend fun putValue(key: Any, value: String) = Unit
    }
}
