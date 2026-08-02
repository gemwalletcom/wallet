package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.TextAutoSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gemwallet.android.model.masked
import com.gemwallet.android.ui.theme.paddingDefault

private val balanceTextLineHeight = 44.sp
private val balanceTextHeight = 52.dp

data class HideToggle(
    val hidden: Boolean,
    val onToggle: (() -> Unit)? = null,
)

internal val HideToggle?.isHidden: Boolean get() = this?.hidden == true

internal fun HideToggle?.mask(text: String): String = text.masked(isHidden)

@Composable
fun DisplayText(
    text: String,
    modifier: Modifier = Modifier,
    hideToggle: HideToggle? = null,
) {
    val hidden = hideToggle.isHidden
    val balanceTextStyle = if (hidden) {
        MaterialTheme.typography.headlineSmall.copy(lineHeight = balanceTextLineHeight)
    } else {
        MaterialTheme.typography.displaySmall.copy(
            fontSize = 42.sp,
            fontWeight = FontWeight.Medium,
            lineHeight = balanceTextLineHeight,
        )
    }
    val balanceAutoSize = if (hidden) {
        null
    } else {
        TextAutoSize.StepBased(
            minFontSize = 20.sp,
            maxFontSize = 42.sp,
            stepSize = 1.sp,
        )
    }
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(balanceTextHeight),
        contentAlignment = Alignment.Center,
    ) {
        val content: @Composable (Modifier) -> Unit = { innerModifier ->
            Text(
                modifier = innerModifier,
                text = hideToggle.mask(text),
                overflow = TextOverflow.MiddleEllipsis,
                maxLines = 1,
                style = balanceTextStyle,
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
                autoSize = balanceAutoSize,
            )
        }
        hideToggle?.onToggle?.let { onToggle ->
            val haptic = LocalHapticFeedback.current
            Surface(
                onClick = {
                    haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                    onToggle()
                },
                shape = CircleShape,
                color = if (hidden) MaterialTheme.colorScheme.background else Color.Transparent,
                contentColor = MaterialTheme.colorScheme.primary,
            ) { content(Modifier.padding(horizontal = paddingDefault)) }
        } ?: content(Modifier)
    }
}
