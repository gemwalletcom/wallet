package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun CenteredDescriptionText(
    text: String,
    modifier: Modifier = Modifier,
) {
    val horizontalPadding = adaptivePadding(default = paddingSmall, compact = 0.dp)

    Text(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = horizontalPadding),
        text = text,
        textAlign = TextAlign.Center,
        color = MaterialTheme.colorScheme.secondary,
    )
}
