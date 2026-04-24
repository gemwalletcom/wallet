package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.GemPlatformErrors
import com.gemwallet.android.model.GemNetworkError
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GatewayException

class ConfirmErrorMapperTest {

    @Test
    fun preloadMapsGatewayNetworkExceptionToNetworkError() {
        val error = GatewayException.NetworkException("Network error: offline")
            .toPreloadConfirmError(Chain.Bitcoin)

        assertTrue(error is ConfirmError.NetworkError)
        assertEquals(GemNetworkError.Display("Network error: offline"), (error as ConfirmError.NetworkError).error)
    }

    @Test
    fun preloadKeepsDustErrorMapping() {
        val error = GatewayException.PlatformException(GemPlatformErrors.Dust.message)
            .toPreloadConfirmError(Chain.Bitcoin)

        assertTrue(error is ConfirmError.DustThreshold)
    }

    @Test
    fun broadcastMapsGatewayNetworkExceptionToNetworkError() {
        val error = GatewayException.NetworkException("Network error: offline")
            .toBroadcastConfirmError()

        assertTrue(error is ConfirmError.NetworkError)
        assertEquals(GemNetworkError.Display("Network error: offline"), (error as ConfirmError.NetworkError).error)
    }

    @Test
    fun broadcastKeepsExistingConfirmError() {
        val source = ConfirmError.SignFail

        assertSame(source, source.toBroadcastConfirmError())
    }
}
