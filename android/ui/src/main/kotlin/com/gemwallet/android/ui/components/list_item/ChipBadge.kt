package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6

@Composable
fun ChipBadge(text: String) {
    Text(
        modifier = Modifier
            .background(
                color = MaterialTheme.colorScheme.surfaceContainerHighest,
                shape = RoundedCornerShape(space6),
            )
            .padding(horizontal = space6, vertical = space2),
        text = text,
        color = MaterialTheme.colorScheme.outline,
        style = MaterialTheme.typography.bodyMedium,
    )
}
