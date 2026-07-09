package com.gemwallet.android.ui.components.chart

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.models.chart.ChartHeaderUIModel
import com.gemwallet.android.ui.models.chart.ChartValueType
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space8

@Composable
fun ChartHeader(
    model: ChartHeaderUIModel,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        model.headerValueText?.let { headerValue ->
            Text(
                text = headerValue,
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
        val changeStyle = if (model.headerValueText != null) {
            MaterialTheme.typography.titleMedium.copy(fontSize = 17.sp, fontWeight = FontWeight.Medium)
        } else {
            MaterialTheme.typography.headlineSmall
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = model.priceText,
                style = changeStyle,
                color = when (model.type) {
                    ChartValueType.Price -> MaterialTheme.colorScheme.onSurface
                    ChartValueType.PriceChange -> model.direction.color()
                },
            )
            model.changeText?.let { change ->
                Spacer(modifier = Modifier.width(space8))
                Text(
                    text = change,
                    style = if (model.headerValueText != null) changeStyle else MaterialTheme.typography.bodyLarge,
                    color = model.direction.color(),
                )
            }
        }
        Box(
            modifier = Modifier.height(paddingDefault),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = model.dateText.orEmpty(),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.secondary,
            )
        }
    }
}
