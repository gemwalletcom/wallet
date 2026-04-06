package com.gemwallet.android.features.settings.networks.viewmodels

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class NodeUrlValidationTest {

    @Test
    fun `parse accepts http and https urls with a host`() {
        assertNotNull(NodeUrlParser.parse("https://rpc.example.com"))
        assertNotNull(NodeUrlParser.parse("http://127.0.0.1:8545"))
    }

    @Test
    fun `parse accepts bare hosts by resolving https url`() {
        assertNotNull(NodeUrlParser.parse("rpc.example.com"))
        assertEquals("https://rpc.example.com", NodeUrlParser.parse("rpc.example.com"))
    }

    @Test
    fun `parse expects trimmed input from caller`() {
        assertEquals("https://rpc.example.com", NodeUrlParser.parse("rpc.example.com"))
        assertEquals("http://127.0.0.1:8545", NodeUrlParser.parse("http://127.0.0.1:8545"))
    }

    @Test
    fun `parse rejects unsupported schemes malformed urls and missing host`() {
        assertNull(NodeUrlParser.parse("ws://rpc.example.com"))
        assertNull(NodeUrlParser.parse("https:///missing-host"))
        assertNull(NodeUrlParser.parse("not-a-url"))
        assertNull(NodeUrlParser.parse(""))
    }
}
