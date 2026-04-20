package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.theme.paddingDefault

private const val BALANCE_MASK = "✱✱✱✱✱"

data class HideToggle(
    val hidden: Boolean,
    val onToggle: () -> Unit,
)

internal val HideToggle?.isHidden: Boolean get() = this?.hidden == true

internal fun HideToggle?.mask(text: String): String = if (isHidden) BALANCE_MASK else text

@Composable
fun DisplayText(
    text: String,
    modifier: Modifier = Modifier,
    hideToggle: HideToggle? = null,
) {
    val hidden = hideToggle.isHidden
    Box(
        modifier = modifier.fillMaxWidth(),
        contentAlignment = Alignment.Center,
    ) {
        val content: @Composable (Modifier) -> Unit = { innerModifier ->
            Text(
                modifier = innerModifier,
                text = hideToggle.mask(text),
                overflow = TextOverflow.MiddleEllipsis,
                maxLines = 1,
                style = (if (hidden) MaterialTheme.typography.headlineSmall
                else MaterialTheme.typography.displaySmall).copy(lineHeight = 44.sp),
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
            )
        }
        hideToggle?.let { toggle ->
            val haptic = LocalHapticFeedback.current
            Surface(
                onClick = {
                    haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                    toggle.onToggle()
                },
                shape = CircleShape,
                color = if (hidden) MaterialTheme.colorScheme.background else Color.Transparent,
                contentColor = MaterialTheme.colorScheme.primary,
            ) { content(Modifier.padding(horizontal = paddingDefault)) }
        } ?: content(Modifier)
    }
}
