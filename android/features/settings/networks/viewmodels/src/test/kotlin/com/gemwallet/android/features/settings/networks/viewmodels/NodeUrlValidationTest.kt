package com.gemwallet.android.features.settings.networks.viewmodels

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NodeUrlValidationTest {

    @Test
    fun `isValidNodeUrl accepts http and https urls with a host`() {
        assertTrue("https://rpc.example.com".isValidNodeUrl())
        assertTrue("http://127.0.0.1:8545".isValidNodeUrl())
    }

    @Test
    fun `isValidNodeUrl rejects unsupported schemes malformed urls and missing host`() {
        assertFalse("ws://rpc.example.com".isValidNodeUrl())
        assertFalse("https:///missing-host".isValidNodeUrl())
        assertFalse("not-a-url".isValidNodeUrl())
        assertFalse("".isValidNodeUrl())
    }
}
