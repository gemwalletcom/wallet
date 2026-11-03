package com.gemwallet.android.ui.integration

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemImageStyle
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.style.badgeImage
import com.gemwallet.android.ui.style.listItemImage
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemInfoAmount
import uniffi.gemstone.GemInfoDescription
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemNumberUnit

@RunWith(AndroidJUnit4::class)
class InfoSheetEntityTest {

    @Test
    fun theWatchWalletSheetShowsTheStandaloneWatchIcon() {
        val image = GemInfoTopic.WatchWallet.infoSheet().sheet.image

        assertEquals(ListItemImage.Drawable(R.drawable.watch_badge, style = ListItemImageStyle.Avatar), image.listItemImage())
        assertNull(image.badgeImage())
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
        val amount = mockGemFormattedNumber(value = 0.002, unit = GemNumberUnit.Symbol("ETH"))
        val fiat = mockGemFormattedNumber(value = 4.0, unit = GemNumberUnit.Currency("USD"))

        assertEquals("${amount.text()} (~${fiat.text()})", GemInfoAmount(amount, fiat).text())
        assertEquals(amount.text(), GemInfoAmount(amount, null).text())
    }
}
