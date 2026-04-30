package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.theme.Spacer6
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6

@Composable
internal fun LatencyStatusBadge(
    latency: ULong?,
    isLoading: Boolean,
) {
    if (isLoading) {
        Spacer6()
        CircularProgressIndicator14()
        return
    }

    val color = latency?.statusColor() ?: MaterialTheme.colorScheme.error
    Row(
        Modifier
            .padding(start = paddingHalfSmall)
            .background(color = color.copy(alpha = alpha10), shape = RoundedCornerShape(space6)),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            modifier = Modifier.padding(
                start = paddingHalfSmall,
                top = space2,
                end = paddingHalfSmall,
                bottom = space2,
            ),
            text = latency?.let { stringResource(R.string.common_latency_in_ms, it.toLong()) }
                ?: stringResource(R.string.errors_error),
            color = color,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.labelMedium,
        )
    }
}

@Composable
private fun ULong.statusColor(): Color {
    return when {
        this < 1024UL -> MaterialTheme.colorScheme.tertiary
        this < 2048UL -> Color(0xffff9314)
        else -> MaterialTheme.colorScheme.error
    }
}
