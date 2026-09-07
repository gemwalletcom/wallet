package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextInputSelection
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.text.TextRange
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test

class AmountInputTest {
    @get:Rule
    val composeRule = createComposeRule()

    private var amount by mutableStateOf("")
    private lateinit var swapAmount: TextFieldState

    @Test
    fun acceptsDecimalInput() {
        setContent()
        listOf("0.", ".5", "12,5", "1234.56", "١٢.٥", "0.000000000000000001").forEach { input ->
            fields.forEach { composeRule.onNodeWithTag(it).performTextReplacement(input) }
            assertAmounts(input)
        }
    }

    @Test
    fun rejectsMalformedReplacementWithoutChangingAmount() {
        setContent("6")
        listOf("6-3", "-1", "1e3", "1.2.3", "1,234.56", "1 234", "$12", "abc").forEach { input ->
            fields.forEach { composeRule.onNodeWithTag(it).performTextReplacement(input) }
            assertAmounts("6")
        }
    }

    @Test
    fun rejectsInvalidInsertion() {
        setContent("6.3")
        listOf("-", ".", ",").forEach { input ->
            fields.forEach { composeRule.onNodeWithTag(it).performTextInput(input) }
            assertAmounts("6.3")
        }
    }

    @Test
    fun allowsSelectionReplacement() {
        setContent("12.5")
        fields.forEach {
            composeRule.onNodeWithTag(it).performTextInputSelection(TextRange(0, 2))
            composeRule.onNodeWithTag(it).performTextInput("3")
        }
        assertAmounts("3.5")
    }

    @Test
    fun allowsClearingAmount() {
        setContent("12.5")
        fields.forEach { composeRule.onNodeWithTag(it).performTextClearance() }
        assertAmounts("")
    }

    @Test
    fun allowsEditingProgrammaticAmount() {
        setContent("6")
        composeRule.runOnIdle {
            amount = "0.0001"
            swapAmount.setTextAndPlaceCursorAtEnd("0.0001")
        }
        fields.forEach { composeRule.onNodeWithTag(it).performTextInput("5") }
        assertAmounts("0.00015")
    }

    private fun setContent(initialAmount: String = "") {
        amount = initialAmount
        swapAmount = TextFieldState(initialAmount)
        composeRule.setContent {
            WalletTheme {
                Column {
                    AmountField(
                        modifier = Modifier.testTag("amount"),
                        amount = amount,
                        assetSymbol = "ETH",
                        currency = Currency.USD,
                        equivalent = "",
                        error = "",
                        onValueChange = { amount = it },
                        onNext = {},
                    )
                    BasicTextField(
                        modifier = Modifier.testTag("swapAmount"),
                        state = swapAmount,
                        inputTransformation = AmountInputTransformation,
                    )
                }
            }
        }
    }

    private fun assertAmounts(expected: String) {
        composeRule.runOnIdle {
            assertEquals(expected, amount)
            assertEquals(expected, swapAmount.text.toString())
        }
    }

    private val fields = listOf("amount", "swapAmount")
}
