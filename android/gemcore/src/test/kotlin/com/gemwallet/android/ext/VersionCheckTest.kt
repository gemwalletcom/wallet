package com.gemwallet.android.ext

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class VersionCheckTest {

    @Test
    fun `returns true for newer dotted versions`() {
        assertTrue(VersionCheck.isVersionHigher(new = "1.2.3", current = "1.0.0"))
        assertTrue(VersionCheck.isVersionHigher(new = "2.1.3.4", current = "2.1.3"))
        assertTrue(VersionCheck.isVersionHigher(new = "0.1", current = "0.0.1"))
        assertTrue(VersionCheck.isVersionHigher(new = "2.27", current = "2.0.27"))
    }

    @Test
    fun `returns false for older or equal dotted versions`() {
        assertFalse(VersionCheck.isVersionHigher(new = "1.0.0", current = "1.2.3"))
        assertFalse(VersionCheck.isVersionHigher(new = "1.2.3", current = "1.2.3"))
        assertFalse(VersionCheck.isVersionHigher(new = "2.1.3", current = "2.1.3.4"))
        assertFalse(VersionCheck.isVersionHigher(new = "1", current = "2"))
    }

    @Test
    fun `returns false when backend version is lower even if string differs`() {
        assertFalse(VersionCheck.isVersionHigher(new = "1.3.100", current = "2.0.12"))
    }
}
