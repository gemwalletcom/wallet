package com.gemwallet.android.ui.integration

import androidx.compose.ui.test.assertHasNoClickAction
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemCopyKind
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemRowAction
import uniffi.gemstone.GemRowMenuItem

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

        composeRule.onNodeWithText(VALUE).assertHasNoClickAction().performClick()

        assertEquals(null, selected)
    }

    @Test
    fun anAddressRowOpensTheAddressOnTap() {
        var opened: String? = null
        composeRule.setContent {
            WalletTheme {
                GemListRowView(row = CONTRACT_ROW, listPosition = ListPosition.Single, onSelectAddress = { opened = it })
            }
        }

        composeRule.onNodeWithText(CONTRACT).performClick()

        assertEquals(CONTRACT, opened)
    }

    @Test
    fun anAddressRowOpensItsMenuOnLongPress() {
        var opened: String? = null
        composeRule.setContent {
            WalletTheme {
                GemListRowView(row = CONTRACT_ROW, listPosition = ListPosition.Single, onSelectAddress = { opened = it })
            }
        }

        composeRule.onNodeWithText(CONTRACT).performTouchInput { longClick() }

        composeRule.onNodeWithText("Copy").assertIsDisplayed()
        assertEquals(null, opened)
    }

    private companion object {
        const val VALUE = "$2.77"
        const val CONTRACT = "0xdAC17F958D2ee523a2206206994597C13D831ec7"
        val CONTRACT_COPY = GemCopy(kind = GemCopyKind.Address("ethereum"), value = CONTRACT, display = CONTRACT)
        val CONTRACT_ROW = GemListRow.Identifier(GemListRowTitle.CONTRACT, CONTRACT_COPY, null, CONTRACT, listOf(GemRowMenuItem.Copy(CONTRACT_COPY)))
    }
}
