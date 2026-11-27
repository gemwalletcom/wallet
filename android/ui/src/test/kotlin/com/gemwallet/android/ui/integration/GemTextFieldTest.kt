package com.gemwallet.android.ui.integration

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.text.TextRange
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.SuffixTextField
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class GemTextFieldTest {
    @get:Rule
    val composeRule = createComposeRule()

    private var value by mutableStateOf("")
    private val changes = mutableListOf<String>()

    @Test
    fun forwardsTypedText() {
        setContent("12")

        field().performTextInput("3")

        assertEquals(listOf("123"), changes)
        assertField("123", TextRange(3))
    }

    @Test
    fun showsTheValueTheCallerKeeps() {
        setContent("12") { it.filter(Char::isDigit) }

        field().performTextInput("a")

        assertEquals(listOf("12a"), changes)
        assertField("12", TextRange(2))
    }

    @Test
    fun valueSetByTheCallerPlacesTheCursorAtTheEnd() {
        setContent("12")

        value = "4567"

        assertField("4567", TextRange(4))
        assertEquals(emptyList<String>(), changes)
    }

    @Test
    fun suffixFieldShowsTheValueTheCallerKeeps() {
        value = "1"
        composeRule.setContent {
            WalletTheme {
                SuffixTextField(value = value, onValueChange = { changes += it; value = it.take(2) }, suffix = "%")
            }
        }

        field().performTextInput("23")

        assertEquals(listOf("123"), changes)
        assertField("12", TextRange(2))
    }

    private fun setContent(initial: String, keep: (String) -> String = { it }) {
        value = initial
        composeRule.setContent {
            WalletTheme {
                GemTextField(value = value, onValueChange = { changes += it; value = keep(it) }, label = "Name")
            }
        }
    }

    private fun field() = composeRule.onNode(hasSetTextAction())

    private fun assertField(text: String, selection: TextRange) {
        composeRule.waitForIdle()
        val config = field().fetchSemanticsNode().config
        assertEquals(text, config[SemanticsProperties.EditableText].text)
        assertEquals(selection, config[SemanticsProperties.TextSelectionRange])
    }
}
