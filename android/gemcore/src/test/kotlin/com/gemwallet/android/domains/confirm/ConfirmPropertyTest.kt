package com.gemwallet.android.domains.confirm

import org.junit.Assert.assertEquals
import org.junit.Test

class ConfirmPropertyTest {

    @Test fun emptyMemo() {
        assertEquals("test memo", ConfirmProperty.Memo("test memo").data)
        assertEquals("-", ConfirmProperty.Memo("").data)
    }
}
