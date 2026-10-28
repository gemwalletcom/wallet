package com.gemwallet.android.ui.integration

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberUnit

@RunWith(AndroidJUnit4::class)
class GemstoneTextTest {

    @Test
    fun feeRate_readsTheCoreRateWithItsLocalizedUnit() {
        val context = mockk<Context> {
            every { getString(R.string.fee_rate_satvB) } returns "sat/vB"
            every { getString(R.string.fee_rate_gwei) } returns "gwei"
        }
        val rate = mockGemFormattedNumber(value = 2.5, unit = GemNumberUnit.Plain)
        val sol = mockGemFormattedNumber(value = 2.5, unit = GemNumberUnit.Symbol(symbol = "SOL"))

        assertEquals("${rate.text()} sat/vB", GemLocalizedText.FeeRate(rate, FeeUnitType.SAT_VB).string(context))
        assertEquals("${rate.text()} gwei", GemLocalizedText.FeeRate(rate, FeeUnitType.GWEI).string(context))
        assertEquals(sol.text(), GemLocalizedText.FeeRate(sol, FeeUnitType.NATIVE).string(context))
    }
}
