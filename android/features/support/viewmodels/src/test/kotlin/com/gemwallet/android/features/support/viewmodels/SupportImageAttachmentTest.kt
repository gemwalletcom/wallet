package com.gemwallet.android.features.support.viewmodels

import org.junit.Assert.assertEquals
import org.junit.Test

class SupportImageAttachmentTest {

    @Test
    fun anImageWithinTheLimitKeepsItsSize() {
        assertEquals(2048 to 1536, supportImageSize(2048, 1536, 2048))
        assertEquals(640 to 480, supportImageSize(640, 480, 2048))
    }

    @Test
    fun aLargePhotoIsScaledSoItsLongestSideIsTheLimit() {
        assertEquals(2048 to 1536, supportImageSize(12_000, 9_000, 2048))
        assertEquals(1536 to 2048, supportImageSize(9_000, 12_000, 2048))
    }
}
