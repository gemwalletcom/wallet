package com.gemwallet.android.ext

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemServiceException

class ServiceErrorTest {

    @Test
    fun coreServiceErrorsReadAsTheirMessageAndStorageAsUnknown() {
        assertEquals(
            GemErrorText.Message("Rewards are not enabled for this user"),
            GemServiceException.Api("Rewards are not enabled for this user").errorText(),
        )
        assertEquals(GemErrorText.Unknown, GemServiceException.Store("disk full").errorText())
    }

    @Test
    fun aCancelledServiceCallReadsAsTheCancelledText() {
        assertEquals(GemErrorText.Cancelled, GemServiceException.Cancelled().errorText())
    }

    @Test
    fun otherErrorsKeepTheirOwnMessage() {
        assertEquals(GemErrorText.Message("offline"), IllegalStateException("offline").errorText())
    }
}
