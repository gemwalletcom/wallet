package com.gemwallet.android.ui.integration

import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemRowAction

@RunWith(AndroidJUnit4::class)
class GemListRowViewActionTest {

    @get:Rule
    val composeRule = createComposeRule()

    @Test
    fun aRowWhoseScreenGivesItAnActionOpensItOnTap() {
        var selected: GemRowAction? = null
        composeRule.setContent {
            WalletTheme {
                GemListRowView(
                    row = GemListRow.Text(GemListRowTitle.PRICE, VALUE),
                    listPosition = ListPosition.Single,
                    action = GemRowAction.Price,
                    onSelect = { selected = it },
                )
            }
        }

        composeRule.onNodeWithText(VALUE).performClick()

        assertEquals(GemRowAction.Price, selected)
    }

    @Test
    fun aRowWithoutAnActionIgnoresTheTap() {
        var selected: GemRowAction? = null
        composeRule.setContent {
            WalletTheme {
                GemListRowView(
                    row = GemListRow.Text(GemListRowTitle.PRICE, VALUE),
                    listPosition = ListPosition.Single,
                    onSelect = { selected = it },
                )
            }
        }

        composeRule.onNodeWithText(VALUE).performClick()

        assertEquals(null, selected)
    }

    private companion object {
        const val VALUE = "$2.77"
    }
}
