package com.gemwallet.android.ui.components

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockFormattedNumber
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.style.badgeIconModel
import com.gemwallet.android.ui.style.iconModel
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemInfoAmount
import uniffi.gemstone.GemInfoDescription
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemNumberUnit

class InfoSheetEntityTest {

    @Test
    fun theWatchWalletSheetShowsTheStandaloneWatchIcon() {
        val image = GemInfoTopic.WatchWallet.infoSheet().sheet.image

        assertEquals(R.drawable.watch_badge, image.iconModel())
        assertNull(image.badgeIconModel())
    }

    @Test
    fun aMemoWarningBoldsTheSymbol() {
        val context = mockk<Context> {
            every { getString(R.string.errors_scan_transaction_memo_required, "**XRP**") } returns "Memo XRP"
        }

        assertEquals("Memo XRP", GemInfoDescription.MemoRequired("XRP").string(context))
    }

    @Test
    fun aRequiredAmountReadsWithTheFiatItIsWorthWhenPriced() {
        val amount = mockFormattedNumber(0.002, GemNumberUnit.Symbol("ETH"))
        val fiat = mockFormattedNumber(4.0, GemNumberUnit.Currency("USD"))

        assertEquals("${amount.text()} (~${fiat.text()})", GemInfoAmount(amount, fiat).text())
        assertEquals(amount.text(), GemInfoAmount(amount, null).text())
    }
}
