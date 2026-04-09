package com.gemwallet.android.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
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
    val clickModifier = hideToggle?.let { toggle ->
        val haptic = LocalHapticFeedback.current
        Modifier
            .clip(CircleShape)
            .clickable {
                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                toggle.onToggle()
            }
    } ?: Modifier
    Box(
        modifier = modifier.fillMaxWidth(),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            modifier = if (hidden) {
                Modifier
                    .then(clickModifier)
                    .background(
                        color = MaterialTheme.colorScheme.background,
                        shape = CircleShape,
                    )
                    .padding(horizontal = paddingDefault)
            } else {
                Modifier.then(clickModifier)
            },
            text = hideToggle.mask(text),
            overflow = TextOverflow.MiddleEllipsis,
            maxLines = 1,
            style = (if (hidden) MaterialTheme.typography.headlineSmall
            else MaterialTheme.typography.displaySmall).copy(lineHeight = 44.sp),
            color = MaterialTheme.colorScheme.onSurface,
            textAlign = TextAlign.Center,
        )
    }
}
