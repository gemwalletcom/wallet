package com.gemwallet.android.ui.components.screen

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.layout.padding
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8
import com.gemwallet.android.ui.theme.space10

@Composable
fun PhraseLayout(
    words: List<String>,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        val halfSize = words.size / 2
        for (i in 0 until halfSize) {
            Row(
                modifier = Modifier.fillMaxWidth()
            ) {
                PhraseWordItem(
                    index = i,
                    word = words[i],
                    isNextToEnter = (i > 0 && words[i].isEmpty() && words[i - 1].isNotEmpty()) ||  (words[i].isEmpty() && i == 0),
                )
                Spacer(modifier = Modifier.width(space8))
                PhraseWordItem(
                    index = i + halfSize,
                    word = words[i + halfSize],
                    isNextToEnter = words[i + halfSize].isEmpty() && words[(i + halfSize) - 1].isNotEmpty(),
                )
            }
            Spacer(modifier = Modifier.height(space8))
        }
    }
}

@Composable
fun RowScope.PhraseWordItem(
    index: Int,
    word: String,
    isNextToEnter: Boolean,
) {
    val verticalPadding = adaptivePadding(default = space10, compact = space6)

    Surface(
        modifier = Modifier.weight(0.5f),
        shadowElevation = 1.dp,
        shape = RoundedCornerShape(space10),
        color = MaterialTheme.colorScheme.background,
        border = if (isNextToEnter) BorderStroke(space2, MaterialTheme.colorScheme.primary) else null
    ) {
        Row(
            modifier = Modifier.padding(horizontal = paddingDefault, vertical = verticalPadding),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "${index + 1}.",
                color = MaterialTheme.colorScheme.secondary,
            )
            Spacer(modifier = Modifier.width(space6))
            Text(
                text = word,
                color = MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.weight(1f),
            )
        }
    }
}
