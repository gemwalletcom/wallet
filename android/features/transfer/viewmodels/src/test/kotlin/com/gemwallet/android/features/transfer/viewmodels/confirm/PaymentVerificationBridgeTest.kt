package com.gemwallet.android.features.transfer.viewmodels.confirm

import org.junit.Assert.assertEquals
import org.junit.Test

class PaymentVerificationBridgeTest {

    @Test
    fun `the form reports completion and failure`() {
        var completed = 0
        var failed = 0
        val bridge = PaymentVerificationBridge(onComplete = { completed += 1 }, onError = { failed += 1 })

        bridge.onDataCollectionComplete("""{"type":"IC_PROGRESS"}""")
        bridge.onDataCollectionComplete("not json")
        assertEquals(0 to 0, completed to failed)

        bridge.onDataCollectionComplete("""{"type":"IC_ERROR"}""")
        assertEquals(0 to 1, completed to failed)

        bridge.onDataCollectionComplete("""{"type":"IC_COMPLETE"}""")
        assertEquals(1 to 1, completed to failed)
    }
}
