package com.gemwallet.android.ui.integration

import android.os.Build
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.screen.PhraseLayout
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config

@RunWith(AndroidJUnit4::class)
class PhraseLayoutSemanticsTest {

    @get:Rule
    val composeRule = createComposeRule()

    private val words = List(12) { "word${it + 1}" }

    private val isSensitive = SemanticsMatcher.expectValue(SemanticsProperties.IsSensitiveData, true)

    private fun setPhrase() {
        composeRule.setContent {
            WalletTheme {
                PhraseLayout(words = words)
            }
        }
    }

    @Test
    @Config(sdk = [Build.VERSION_CODES.UPSIDE_DOWN_CAKE])
    fun theWordsAndTheirNumbersAreMarkedSensitive() {
        setPhrase()

        words.forEachIndexed { index, word ->
            composeRule.onNodeWithTag("word_$index").assert(isSensitive)
            composeRule.onNodeWithText(word).assert(isSensitive)
            composeRule.onNodeWithText("${index + 1}.").assert(isSensitive)
        }
    }

    @Test
    @Config(sdk = [Build.VERSION_CODES.TIRAMISU])
    fun theWordsAreHiddenBeforeAndroid14() {
        setPhrase()

        words.forEachIndexed { index, word ->
            composeRule.onNodeWithText(word).assertDoesNotExist()
            composeRule.onNodeWithText("${index + 1}.").assertDoesNotExist()
        }
    }
}
