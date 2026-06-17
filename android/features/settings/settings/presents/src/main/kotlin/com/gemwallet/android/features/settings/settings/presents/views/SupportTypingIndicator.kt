package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.animation.core.EaseInOut
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.StartOffset
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall

private object TypingDots {
    const val COUNT = 3
    const val DURATION_MS = 500
    const val DELAY_MS = 200
}

@Composable
fun SupportTypingIndicator(name: String) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.Start,
        verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        Text(
            text = name,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.secondary,
        )
        SupportTypingDots(
            modifier = Modifier
                .background(MaterialTheme.colorScheme.surfaceContainerHighest, CircleShape)
                .padding(horizontal = paddingDefault, vertical = paddingMiddle),
        )
    }
}

@Composable
private fun SupportTypingDots(modifier: Modifier = Modifier) {
    val transition = rememberInfiniteTransition(label = "typing")
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        repeat(TypingDots.COUNT) { index ->
            val alpha by transition.animateFloat(
                initialValue = 1f,
                targetValue = alpha50,
                animationSpec = infiniteRepeatable(
                    animation = tween(durationMillis = TypingDots.DURATION_MS, easing = EaseInOut),
                    repeatMode = RepeatMode.Reverse,
                    initialStartOffset = StartOffset(index * TypingDots.DELAY_MS),
                ),
                label = "dot$index",
            )
            Box(
                modifier = Modifier
                    .size(paddingSmall)
                    .background(MaterialTheme.colorScheme.secondary.copy(alpha = alpha), CircleShape),
            )
        }
    }
}
