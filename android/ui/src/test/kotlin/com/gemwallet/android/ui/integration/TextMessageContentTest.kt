package com.gemwallet.android.ui.integration

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.textMessageContent
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TextMessageContentTest {

    @get:Rule
    val composeRule = createComposeRule()

    @Test
    fun aLongMessageIsLaidOutOnlyAsFarAsTheScreenShowsIt() {
        composeRule.setContent {
            WalletTheme {
                LazyColumn { textMessageContent("a".repeat(256 * 1024)) }
            }
        }

        val shown = composeRule.onAllNodes(hasText("aaa", substring = true))
            .fetchSemanticsNodes()
            .map { node -> node.config.getOrNull(SemanticsProperties.Text).orEmpty().sumOf { it.length } }

        assertTrue("shown blocks: $shown", shown.isNotEmpty() && shown.size < 16 && shown.all { it <= 1024 })
    }
}
