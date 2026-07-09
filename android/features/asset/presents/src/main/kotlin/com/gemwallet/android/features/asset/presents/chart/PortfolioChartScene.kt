package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset.viewmodels.chart.models.portfolioChartHeader
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.PortfolioChartViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.PortfolioChartType
import com.wallet.core.primitives.PortfolioType

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PortfolioChartScene(
    onCancel: () -> Unit,
    viewModel: PortfolioChartViewModel = hiltViewModel(),
) {
    val statistics by viewModel.statistics.collectAsStateWithLifecycle()
    val currency by viewModel.currency.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val selectedType by viewModel.selectedType.collectAsStateWithLifecycle()
    val showSegmentedControl by viewModel.showSegmentedControl.collectAsStateWithLifecycle()
    val selectedChartType by viewModel.selectedChartType.collectAsStateWithLifecycle()
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()
    val showChartTypePicker = selectedType == PortfolioType.Perpetuals
    val pullToRefreshState = rememberPullToRefreshState()

    Scene(
        titleContent = {
            if (showSegmentedControl) {
                PortfolioTypeSelector(selected = selectedType, onSelect = viewModel::setType)
            } else {
                Text(stringResource(selectedType.titleRes()))
            }
        },
        onClose = onCancel,
        actions = {
            if (showChartTypePicker) {
                ChartTypeSelector(selected = selectedChartType, onSelect = viewModel::setChartType)
            }
        },
    ) {
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = viewModel::refresh,
            state = pullToRefreshState,
            indicator = {
                PullToRefreshDefaults.Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isRefreshing,
                    state = pullToRefreshState,
                )
            },
        ) {
            LazyColumn {
                item { PortfolioChart(viewModel) }
                if (state.chart is StateViewType.Data || state.chart == StateViewType.NoData) {
                    portfolioStatistics(currency, statistics)
                }
            }
        }
    }
}

@Composable
private fun PortfolioTypeSelector(selected: PortfolioType, onSelect: (PortfolioType) -> Unit) {
    TabsBar(PortfolioType.entries, selected, onSelect) { type ->
        Text(stringResource(type.titleRes()))
    }
}

@Composable
private fun ChartTypeSelector(selected: PortfolioChartType, onSelect: (PortfolioChartType) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    TextButton(onClick = { expanded = true }) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = stringResource(selected.titleRes()),
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.SemiBold,
            )
            Icon(
                imageVector = AppIcons.ExpandMore,
                tint = MaterialTheme.colorScheme.onSurface,
                contentDescription = "select_chart_type",
            )
        }
    }
    DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
        PortfolioChartType.entries.forEach { type ->
            DropdownMenuItem(
                text = { Text(stringResource(type.titleRes())) },
                onClick = {
                    onSelect(type)
                    expanded = false
                },
            )
        }
    }
}

@Composable
private fun PortfolioChart(viewModel: PortfolioChartViewModel) {
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()
    val periods by viewModel.availablePeriods.collectAsStateWithLifecycle()

    ChartSection(
        state = state,
        onPeriodSelect = viewModel::setPeriod,
        periods = periods,
    ) { uiModel, selectedPoint -> portfolioChartHeader(uiModel, selectedPoint) }
}

private fun PortfolioType.titleRes(): Int = when (this) {
    PortfolioType.Wallet -> R.string.wallet_portfolio_title
    PortfolioType.Perpetuals -> R.string.perpetuals_title
}

private fun PortfolioChartType.titleRes(): Int = when (this) {
    PortfolioChartType.Value -> R.string.perpetual_value
    PortfolioChartType.Pnl -> R.string.perpetual_pnl
}
