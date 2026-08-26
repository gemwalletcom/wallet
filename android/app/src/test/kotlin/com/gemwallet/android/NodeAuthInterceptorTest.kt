package com.gemwallet.android

import com.gemwallet.android.data.services.gemapi.DeviceToken
import com.gemwallet.android.serializer.toJson
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import okhttp3.Interceptor
import okhttp3.Request
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemPreferencesStore

class NodeAuthInterceptorTest {
    private val preferences = mockk<GemPreferencesStore>()

    @Test
    fun interceptAddsValidTokenOnlyToGemNodes() {
        every { preferences.get(any()) } returns DeviceToken("valid", 200u).toJson()
        val interceptor = NodeAuthInterceptor(preferences, nodesDomain = "gemnodes.com") { 100u }

        val nodeChain = mockk<Interceptor.Chain>()
        val nodeRequest = slot<Request>()
        every { nodeChain.request() } returns Request.Builder().url("https://asia.gemnodes.com/ethereum").build()
        every { nodeChain.proceed(capture(nodeRequest)) } returns mockk()
        interceptor.intercept(nodeChain)
        assertEquals("Bearer valid", nodeRequest.captured.header("Authorization"))

        val customChain = mockk<Interceptor.Chain>()
        val customRequest = slot<Request>()
        every { customChain.request() } returns Request.Builder().url("https://rpc.example.com").build()
        every { customChain.proceed(capture(customRequest)) } returns mockk()
        interceptor.intercept(customChain)
        assertNull(customRequest.captured.header("Authorization"))

        every { preferences.get(any()) } returns DeviceToken("expired", 100u).toJson()
        val expiredChain = mockk<Interceptor.Chain>()
        val expiredRequest = slot<Request>()
        every { expiredChain.request() } returns Request.Builder().url("https://gemnodes.com/bitcoin").build()
        every { expiredChain.proceed(capture(expiredRequest)) } returns mockk()
        interceptor.intercept(expiredChain)
        assertNull(expiredRequest.captured.header("Authorization"))
    }
}
