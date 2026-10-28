package com.gemwallet.android.ui.components.screen

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space1
import com.gemwallet.android.ui.theme.space10
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8
import uniffi.gemstone.GemSecretPhraseRow
import uniffi.gemstone.secretPhraseRows

@Composable
fun PhraseLayout(words: List<String>, modifier: Modifier = Modifier, highlightIndex: Int? = null) {
    val rows = remember(words.size) { secretPhraseRows(words.size) }
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        for (row in rows) {
            when (row) {
                is GemSecretPhraseRow.Pair -> Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(space8),
                ) {
                    PhraseWordItem(row.left, words[row.left], isHighlighted = row.left == highlightIndex, modifier = Modifier.weight(1f))
                    PhraseWordItem(row.right, words[row.right], isHighlighted = row.right == highlightIndex, modifier = Modifier.weight(1f))
                }

                is GemSecretPhraseRow.Single -> PhraseWordItem(row.index, words[row.index], isHighlighted = row.index == highlightIndex)
            }
            Spacer(modifier = Modifier.height(space8))
        }
    }
}

@Composable
private fun PhraseWordItem(index: Int, word: String, isHighlighted: Boolean, modifier: Modifier = Modifier) {
    val verticalPadding = adaptivePadding(default = space10, compact = space6)

    Surface(
        modifier = modifier,
        shadowElevation = space1,
        shape = RoundedCornerShape(space10),
        color = MaterialTheme.colorScheme.background,
        border = if (isHighlighted) BorderStroke(space2, MaterialTheme.colorScheme.primary) else null,
    ) {
        Row(
            modifier = Modifier.padding(horizontal = paddingDefault, vertical = verticalPadding),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "${index + 1}.",
                color = MaterialTheme.colorScheme.secondary,
            )
            Spacer(modifier = Modifier.width(space6))
            Text(
                text = word,
                modifier = Modifier.testTag("word_$index"),
                color = MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}
