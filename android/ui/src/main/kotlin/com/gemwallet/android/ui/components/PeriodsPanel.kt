package com.gemwallet.android.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.ChartPeriod

@Composable
fun PeriodsPanel(
    period: ChartPeriod,
    onSelect: (ChartPeriod) -> Unit
) {
    Row(
        modifier = Modifier.padding(start = paddingDefault, end = paddingDefault, bottom = paddingDefault),
        horizontalArrangement = Arrangement.spacedBy(0.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ChartPeriod.entries.forEach {
            val label = it.getLabel() ?: return@forEach
            PeriodButton(label, it == period) { onSelect(it) }
        }
    }
}

@Composable
private fun RowScope.PeriodButton(title: String, isSelected: Boolean, onClick: () -> Unit) {
    val shape = RoundedCornerShape(paddingSmall)
    val bgColor by animateColorAsState(
        targetValue = if (isSelected) MaterialTheme.colorScheme.background else Color.Transparent,
        animationSpec = tween(200),
        label = "periodBg",
    )

    Box(
        modifier = Modifier
            .weight(1f)
            .clip(shape)
            .background(bgColor)
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                onClick = onClick,
            )
            .padding(space6),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Medium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun ChartPeriod.getLabel(): String? {
    val strId = when (this) {
        ChartPeriod.Hour -> R.string.charts_hour
        ChartPeriod.Day -> R.string.charts_day
        ChartPeriod.Week -> R.string.charts_week
        ChartPeriod.Month -> R.string.charts_month
        ChartPeriod.Year -> R.string.charts_year
        ChartPeriod.All -> R.string.charts_all
    }
    return stringResource(id = strId)
}
