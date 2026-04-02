package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PeriodsPanel
import com.gemwallet.android.ui.components.chart.GemLineChart
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.features.asset.viewmodels.chart.models.PricePoint
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.ChartViewModel

private object ChartSceneMetrics {
    val frameHeight = 320.dp
    val dateRowHeight = 16.dp
}

@Composable
fun Chart(
    viewModel: ChartViewModel = hiltViewModel()
) {
    val uiModel by viewModel.chartUIModel.collectAsStateWithLifecycle()
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()

    key(state.period) {
        var selectedIndex by remember { mutableStateOf<Int?>(null) }

        Column {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(ChartSceneMetrics.frameHeight),
            ) {
                val chartPoints = uiModel.chartPoints
                val isReady = !state.loading && state.period == uiModel.period && chartPoints.isNotEmpty()
                val point: PricePoint? = if (!isReady) {
                    null
                } else {
                    val idx = selectedIndex
                    if (idx != null && idx in chartPoints.indices) chartPoints[idx]
                    else uiModel.currentPoint ?: chartPoints.lastOrNull()
                }

                ChartHeader(
                    point = point,
                    isScrubbing = selectedIndex != null && point != null,
                    modifier = Modifier.padding(top = paddingSmall, bottom = space4),
                )

                Box(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth(),
                ) {
                    when {
                        state.loading || state.period != uiModel.period -> {
                            CircularProgressIndicator20(
                                modifier = Modifier.align(Alignment.Center),
                                color = MaterialTheme.colorScheme.primary,
                            )
                        }
                        state.empty -> ChartError()
                        uiModel.renderPoints.size >= 2 -> {
                            GemLineChart(
                                points = uiModel.renderPoints,
                                lineColor = MaterialTheme.colorScheme.primary,
                                selectedIndex = selectedIndex,
                                onSelectionChanged = { selectedIndex = it },
                                minLabel = uiModel.minLabel,
                                maxLabel = uiModel.maxLabel,
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(paddingDefault))
            PeriodsPanel(state.period, viewModel::setPeriod)
        }
    }
}

@Composable
private fun ChartHeader(
    point: PricePoint?,
    isScrubbing: Boolean,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(space4),
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(space4),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = point?.yLabel ?: "",
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
            if (point != null && point.percentage.isNotEmpty()) {
                Text(
                    text = point.percentage,
                    style = MaterialTheme.typography.bodyLarge,
                    color = point.priceState.color(),
                )
            }
        }

        Box(modifier = Modifier.height(ChartSceneMetrics.dateRowHeight), contentAlignment = Alignment.Center) {
            if (isScrubbing && point != null) {
                Text(
                    text = getRelativeDate(point.timestamp),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.secondary,
                )
            }
        }
    }
}

@Composable
fun ChartError() {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .defaultPadding()
    ) {
        Text(
            modifier = Modifier.align(Alignment.Center),
            textAlign = TextAlign.Center,
            text = stringResource(R.string.errors_error_occured),
        )
    }
}
