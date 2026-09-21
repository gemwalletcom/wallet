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
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space8
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.GemValueTone

@Composable
fun ChartHeader(header: GemChartHeader, date: String?, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        val secondaryValue = header.secondaryValue?.text()
        secondaryValue?.let { headerValue ->
            Text(
                text = headerValue,
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
        val changeStyle = if (secondaryValue != null) {
            MaterialTheme.typography.titleMedium.copy(fontSize = 17.sp, fontWeight = FontWeight.Medium)
        } else {
            MaterialTheme.typography.headlineSmall
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = header.value.text(),
                style = changeStyle,
                color = header.value.tone.color(),
            )
            header.change?.let { change ->
                Spacer(modifier = Modifier.width(space8))
                Text(
                    text = change.text(),
                    style = if (secondaryValue != null) changeStyle else MaterialTheme.typography.bodyLarge,
                    color = (change.tone ?: GemValueTone.PLAIN).color(),
                )
            }
        }
        Box(
            modifier = Modifier.height(paddingDefault),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = date.orEmpty(),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.secondary,
            )
        }
    }
}
