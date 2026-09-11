package com.gemwallet.android.ext

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemServiceException

class ServiceErrorTest {

    @Test
    fun coreServiceErrorsReadAsTheirMessage() {
        assertEquals("Rewards are not enabled for this user", GemServiceException.Api("Rewards are not enabled for this user").serviceMessage())
        assertEquals("disk full", GemServiceException.Store("disk full").serviceMessage())
    }

    @Test
    fun otherErrorsKeepTheirOwnMessage() {
        assertEquals("offline", IllegalStateException("offline").serviceMessage())
    }
}
