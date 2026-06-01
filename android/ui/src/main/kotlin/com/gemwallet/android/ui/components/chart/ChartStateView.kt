package com.gemwallet.android.ui.components.chart

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PeriodsPanel
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.chart.ChartHeaderUIModel
import com.gemwallet.android.ui.models.chart.ChartViewState
import com.gemwallet.android.ui.theme.chartFrameHeight
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space4
import com.wallet.core.primitives.ChartPeriod

@Composable
fun ChartStateView(
    state: ChartViewState,
    header: ChartHeaderUIModel?,
    period: ChartPeriod,
    onPeriodSelect: (ChartPeriod) -> Unit,
    modifier: Modifier = Modifier,
    chartBody: @Composable BoxScope.() -> Unit,
) {
    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().height(chartFrameHeight),
        ) {
            header?.takeIf { state == ChartViewState.Ready }?.let {
                ChartHeader(
                    model = it,
                    modifier = Modifier.padding(top = paddingSmall, bottom = space4),
                )
            }
            Box(modifier = Modifier.fillMaxWidth().weight(1f)) {
                when (state) {
                    ChartViewState.Loading -> CircularProgressIndicator20(
                        modifier = Modifier.align(Alignment.Center),
                        color = MaterialTheme.colorScheme.primary,
                    )
                    ChartViewState.Empty -> EmptyStateView(
                        modifier = Modifier.fillMaxSize(),
                        title = stringResource(R.string.common_not_available),
                        icon = painterResource(R.drawable.empty_activity),
                    )
                    ChartViewState.Error -> EmptyStateView(
                        modifier = Modifier.fillMaxSize(),
                        title = stringResource(R.string.errors_no_data_available),
                        iconVector = AppIcons.Warning,
                    )
                    ChartViewState.Ready -> chartBody()
                }
            }
        }
        PeriodsPanel(period, onPeriodSelect)
    }
}

