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
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import java.text.DecimalFormatSymbols

class AmountInputTest {
    @get:Rule
    val composeRule = createComposeRule()

    private val separator = DecimalFormatSymbols.getInstance().decimalSeparator
    private val other = if (separator == '.') ',' else '.'

    private var amount by mutableStateOf("")
    private lateinit var swapAmount: TextFieldState

    @Test
    fun acceptsDecimalInput() {
        setContent()
        listOf(
            "0$separator",
            "${separator}5",
            "12${separator}5",
            "1234${separator}56",
            "١٢${separator}٥",
            "0${separator}000000000000000001",
        ).forEach { input ->
            fields.forEach { composeRule.onNodeWithTag(it).performTextReplacement(input) }
            assertAmounts(input)
        }
    }

    @Test
    fun typedSeparatorTakesTheOneTheDeviceUses() {
        setContent("12")
        fields.forEach { composeRule.onNodeWithTag(it).performTextInput("$other") }
        assertAmounts("12$separator")
        fields.forEach { composeRule.onNodeWithTag(it).performTextInput("5") }
        assertAmounts("12${separator}5")
    }

    @Test
    fun cleansMalformedReplacement() {
        setContent("6")
        mapOf(
            "12${other}5" to "12${separator}5",
            "1${other}2${other}3" to "1${separator}23",
            "6-3" to "63",
            "-1" to "1",
            "1e3" to "13",
            "1 234" to "1234",
            "$12" to "12",
            "abc" to "",
        ).forEach { (input, expected) ->
            fields.forEach { composeRule.onNodeWithTag(it).performTextReplacement(input) }
            assertAmounts(expected)
        }
    }

    @Test
    fun dropsInvalidInsertion() {
        setContent("6${separator}3")
        listOf("-", ".", ",").forEach { input ->
            fields.forEach { composeRule.onNodeWithTag(it).performTextInput(input) }
            assertAmounts("6${separator}3")
        }
    }

    @Test
    fun allowsSelectionReplacement() {
        setContent("12${separator}5")
        fields.forEach {
            composeRule.onNodeWithTag(it).performTextInputSelection(TextRange(0, 2))
            composeRule.onNodeWithTag(it).performTextInput("3")
        }
        assertAmounts("3${separator}5")
    }

    @Test
    fun allowsClearingAmount() {
        setContent("12${separator}5")
        fields.forEach { composeRule.onNodeWithTag(it).performTextClearance() }
        assertAmounts("")
    }

    @Test
    fun allowsEditingProgrammaticAmount() {
        setContent("6")
        composeRule.runOnIdle {
            amount = "0${separator}0001"
            swapAmount.setTextAndPlaceCursorAtEnd("0${separator}0001")
        }
        fields.forEach { composeRule.onNodeWithTag(it).performTextInput("5") }
        assertAmounts("0${separator}00015")
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
                        symbol = AmountSymbolUIModel("ETH", AmountSymbolPlacement.Trailing),
                        equivalent = "",
                        error = "",
                        onValueChange = { amount = it },
                        onNext = {},
                    )
                    BasicTextField(
                        modifier = Modifier.testTag("swapAmount"),
                        state = swapAmount,
                        inputTransformation = AmountInputTransformation(),
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
