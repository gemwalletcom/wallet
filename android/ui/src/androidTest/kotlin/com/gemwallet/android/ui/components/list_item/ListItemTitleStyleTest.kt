package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextStyle
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test

class ListItemTitleStyleTest {

    @get:Rule
    val composeRule = createComposeRule()

    @Test
    fun aModelRowTitleKeepsTheRegularWeightWithAndWithoutAnImage() {
        lateinit var expected: TextStyle
        composeRule.setContent {
            WalletTheme {
                expected = MaterialTheme.typography.bodyLarge
                Column {
                    ListItem(model = ListItemModel(title = PLAIN, subtitle = "1"), listPosition = ListPosition.First)
                    ListItem(
                        model = ListItemModel(title = IMAGED, subtitle = "2", image = ListItemImage.Drawable(R.drawable.ic_gem_foreground)),
                        listPosition = ListPosition.Last,
                    )
                }
            }
        }

        listOf(PLAIN, IMAGED).forEach { title ->
            val style = composeRule.onNodeWithText(title, useUnmergedTree = true).textStyle()
            assertEquals(title, expected.fontSize, style.fontSize)
            assertEquals(title, expected.fontWeight, style.fontWeight)
        }
    }

    private fun SemanticsNodeInteraction.textStyle(): TextStyle {
        val results = mutableListOf<TextLayoutResult>()
        fetchSemanticsNode().config[SemanticsActions.GetTextLayoutResult].action?.invoke(results)
        return results.first().layoutInput.style
    }

    private companion object {
        const val PLAIN = "Plain title"
        const val IMAGED = "Imaged title"
    }
}
