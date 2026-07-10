package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun PercentSuggestionsBar(
    suggestions: List<Int>,
    modifier: Modifier = Modifier,
    onPercentSelected: (Int) -> Unit,
) {
    SuggestionsBar(
        labels = suggestions.map { "$it%" },
        modifier = modifier,
        onSelected = { index -> onPercentSelected(suggestions[index]) },
    )
}

@Composable
fun SuggestionsBar(
    labels: List<String>,
    modifier: Modifier = Modifier,
    onSelected: (Int) -> Unit,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        labels.forEachIndexed { index, label ->
            SuggestionChip(
                modifier = Modifier.weight(1f),
                onClick = { onSelected(index) },
                label = {
                    Text(
                        text = label,
                        modifier = Modifier.fillMaxWidth(),
                        textAlign = TextAlign.Center,
                    )
                },
            )
        }
    }
}
