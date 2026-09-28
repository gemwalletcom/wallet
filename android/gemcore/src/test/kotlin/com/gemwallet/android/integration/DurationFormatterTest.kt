package com.gemwallet.android.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.domains.duration.formatEstimatedConfirmation
import junit.framework.TestCase.assertEquals
import junit.framework.TestCase.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import java.util.Locale

@RunWith(AndroidJUnit4::class)
class DurationFormatterTest {

    @Test
    fun estimatedConfirmation_readsThePartsWithTheLocaleUnits() {
        assertEquals("≈ 12 min", formatEstimatedConfirmation(720u, Locale.US))
        assertEquals("≈ 1 min, 30 sec", formatEstimatedConfirmation(90u, Locale.US))
        assertEquals("≈ 45 sec", formatEstimatedConfirmation(45u, Locale.US))
        assertEquals("≈ 1 Min., 30 Sek.", formatEstimatedConfirmation(90u, Locale.GERMANY))
        assertNull(formatEstimatedConfirmation(0u, Locale.US))
    }
}
