package com.gemwallet.android.ui.integration

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextInputSelection
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.text.TextRange
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.fields.AmountField
import com.gemwallet.android.ui.components.fields.AmountSymbolPlacement
import com.gemwallet.android.ui.components.fields.AmountSymbolUIModel
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import java.text.DecimalFormatSymbols

@RunWith(AndroidJUnit4::class)
class AmountInputTest {
    @get:Rule
    val composeRule = createComposeRule()

    private val separator = DecimalFormatSymbols.getInstance().decimalSeparator
    private val other = if (separator == '.') ',' else '.'

    private lateinit var amount: TextFieldState

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
            field().performTextReplacement(input)
            assertAmount(input)
        }
    }

    @Test
    fun typedSeparatorTakesTheOneTheDeviceUses() {
        setContent("12")
        field().performTextInput("$other")
        assertAmount("12$separator")
        field().performTextInput("5")
        assertAmount("12${separator}5")
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
            field().performTextReplacement(input)
            assertAmount(expected)
        }
    }

    @Test
    fun dropsInvalidInsertion() {
        setContent("6${separator}3")
        listOf("-", ".", ",").forEach { input ->
            field().performTextInput(input)
            assertAmount("6${separator}3")
        }
    }

    @Test
    fun allowsSelectionReplacement() {
        setContent("12${separator}5")
        field().performTextInputSelection(TextRange(0, 2))
        field().performTextInput("3")
        assertAmount("3${separator}5")
    }

    @Test
    fun allowsClearingAmount() {
        setContent("12${separator}5")
        field().performTextClearance()
        assertAmount("")
    }

    @Test
    fun allowsEditingProgrammaticAmount() {
        setContent("6")
        composeRule.runOnIdle { amount.setTextAndPlaceCursorAtEnd("0${separator}0001") }
        field().performTextInput("5")
        assertAmount("0${separator}00015")
    }

    @Test
    fun showsSymbolBesideTheAmountAndZeroWhileEmpty() {
        setContent()
        composeRule.onNodeWithText("ETH").assertIsDisplayed()
        field().assertTextEquals("0")
        field().performTextInput("7")
        field().assertTextEquals("7")
        assertAmount("7")
    }

    private fun setContent(initialAmount: String = "") {
        amount = TextFieldState(initialAmount)
        composeRule.setContent {
            WalletTheme {
                Column {
                    AmountField(
                        modifier = Modifier.testTag("amount"),
                        state = amount,
                        symbol = AmountSymbolUIModel("ETH", AmountSymbolPlacement.Trailing),
                        equivalent = "",
                        error = "",
                        onNext = {},
                    )
                }
            }
        }
    }

    private fun field() = composeRule.onNodeWithTag("amount")

    private fun assertAmount(expected: String) {
        composeRule.runOnIdle { assertEquals(expected, amount.text.toString()) }
    }
}
