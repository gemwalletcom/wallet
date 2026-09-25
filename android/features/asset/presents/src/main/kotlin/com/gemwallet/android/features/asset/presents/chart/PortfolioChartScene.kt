package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
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
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.PortfolioChartViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.PortfolioType
import uniffi.gemstone.PortfolioChartType

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PortfolioChartScene(onCancel: () -> Unit, viewModel: PortfolioChartViewModel = hiltViewModel()) {
    val refreshIntervalMillis by viewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis, viewModel::refresh)

    val statistics by viewModel.statistics.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val selectedType by viewModel.selectedType.collectAsStateWithLifecycle()
    val showSegmentedControl by viewModel.showSegmentedControl.collectAsStateWithLifecycle()
    val selectedChartType by viewModel.selectedChartType.collectAsStateWithLifecycle()
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()
    val showChartTypePicker by viewModel.showChartTypePicker.collectAsStateWithLifecycle()

    Scene(
        titleContent = {
            if (showSegmentedControl) {
                PortfolioTypeSelector(selected = selectedType, onSelect = viewModel::setType)
            } else {
                Text(stringResource(selectedType.stringRes()))
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
            containerColor = PullToRefreshDefaults.indicatorContainerColor,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                item { PortfolioChart(viewModel) }
                if (state.chart is StateViewType.Data || state.chart == StateViewType.NoData) {
                    if (statistics.isNotEmpty()) {
                        item { SubheaderItem(R.string.common_info) }
                        itemsPositioned(statistics) { position, row -> GemListRowView(row = row, listPosition = position) }
                    }
                }
            }
        }
    }
}

@Composable
private fun PortfolioTypeSelector(selected: PortfolioType, onSelect: (PortfolioType) -> Unit) {
    TabsBar(PortfolioType.entries, selected, onSelect) { type ->
        Text(stringResource(type.stringRes()))
    }
}

@Composable
private fun ChartTypeSelector(selected: PortfolioChartType, onSelect: (PortfolioChartType) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    TextButton(onClick = { expanded = true }) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = stringResource(selected.stringRes()),
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
                text = { Text(stringResource(type.stringRes())) },
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
    )
}
