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
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.features.settings.networks.viewmodels.models.LatencyTone
import com.gemwallet.android.features.settings.networks.viewmodels.models.LatencyUIModel
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.theme.Spacer6
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6

@Composable
internal fun LatencyStatusBadge(latency: LatencyUIModel) {
    val color = when (latency.tone) {
        LatencyTone.Loading -> {
            Spacer6()
            CircularProgressIndicator14()
            return
        }
        LatencyTone.Fast -> MaterialTheme.colorScheme.tertiary
        LatencyTone.Normal -> Color(0xffff9314)
        LatencyTone.Slow, LatencyTone.Error -> MaterialTheme.colorScheme.error
    }
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
            text = latency.text,
            color = color,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.labelMedium,
        )
    }
}
