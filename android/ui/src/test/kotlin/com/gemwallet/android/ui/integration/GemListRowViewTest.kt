package com.gemwallet.android.ui.integration

import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.assertHasNoClickAction
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
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
class GemListRowViewTest {

    @get:Rule
    val composeRule = createComposeRule()

    private val quote = GemListRow.Quote(GemListRowTitle.PRICE, null, null)

    @Test
    fun aRowSelectsTheActionItsCallerCarries() {
        var selected: GemRowAction? = null
        composeRule.setContent {
            WalletTheme {
                GemListRowView(row = quote, listPosition = ListPosition.Single, modifier = Modifier.testTag("row"), action = GemRowAction.Price, onSelect = { selected = it })
            }
        }

        composeRule.onNodeWithTag("row").performClick()

        assertEquals(GemRowAction.Price, selected)
    }

    @Test
    fun aRowWithoutAnActionDoesNotSelect() {
        composeRule.setContent {
            WalletTheme {
                GemListRowView(row = quote, listPosition = ListPosition.Single, modifier = Modifier.testTag("row"), onSelect = {})
            }
        }

        composeRule.onNodeWithTag("row").assertHasNoClickAction()
    }
}
