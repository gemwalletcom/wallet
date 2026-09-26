package com.gemwallet.android.features.perpetuals.presents.perpetuals

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.snapshotFlow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.assets.presents.select.RecentsScreen
import com.gemwallet.android.features.assets.viewmodels.select.RecentsViewModel
import com.gemwallet.android.features.perpetuals.viewmodels.PerpetualsViewModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.wallet.core.primitives.RecentActivityType

@Composable
fun PerpetualsScreen(
    onCancel: () -> Unit,
    onOpenPerpetual: AssetIdAction,
    onOpenPortfolio: () -> Unit,
    amountAction: AmountTransactionAction,
    viewModel: PerpetualsViewModel = hiltViewModel(),
    recentsViewModel: RecentsViewModel = hiltViewModel(),
) {
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val unpinnedPerpetuals by viewModel.unpinnedPerpetuals.collectAsStateWithLifecycle()
    val pinnedPerpetuals by viewModel.pinnedPerpetuals.collectAsStateWithLifecycle()
    val positions by viewModel.positionRows.collectAsStateWithLifecycle()
    val balanceHeader by viewModel.balanceHeader.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val isSearching by viewModel.isSearching.collectAsStateWithLifecycle()
    val query = rememberTextFieldState()

    LaunchedEffect(query) {
        snapshotFlow { query.text.toString() }.collect(viewModel::setQuery)
    }

    LaunchedEffect(Unit) {
        viewModel.refreshMarkets()
    }

    val refreshIntervalMillis by viewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis, viewModel::refreshMarkets)

    DisposableEffect(Unit) {
        viewModel.subscribeMarketPrices()
        onDispose { viewModel.unsubscribeMarketPrices() }
    }

    PerpetualsScene(
        isRefreshing = isRefreshing,
        balanceHeader = balanceHeader,
        unpinnedPerpetuals = unpinnedPerpetuals,
        pinnedPerpetuals = pinnedPerpetuals,
        positions = positions,
        recent = recent,
        query = query,
        sections = sections,
        isSearching = isSearching,
        onAction = { action ->
            when (action) {
                PerpetualsAction.Refresh -> viewModel.onRefresh()

                is PerpetualsAction.SetSearching -> viewModel.setSearching(action.isSearching)

                PerpetualsAction.Close -> onCancel()

                PerpetualsAction.Withdraw -> balanceHeader?.let { amountAction(AmountParams.Withdraw(it.withdrawAsset.toPrimitives().id)) }

                PerpetualsAction.Deposit -> balanceHeader?.let { amountAction(AmountParams.Deposit(it.depositAsset.toPrimitives().id)) }

                PerpetualsAction.OpenPortfolio -> onOpenPortfolio()

                is PerpetualsAction.TogglePin -> viewModel.onTogglePin(action.perpetualId)

                is PerpetualsAction.OpenPerpetual -> {
                    onOpenPerpetual(action.asset.id)
                    viewModel.onOpenPerpetual(action.asset)
                }

                is PerpetualsAction.OpenRecent -> onOpenPerpetual(action.asset.id)

                PerpetualsAction.OpenRecentsSheet -> recentsViewModel.show(types = listOf(RecentActivityType.Perpetual))
            }
        },
    )

    RecentsScreen(
        viewModel = recentsViewModel,
        onSelect = { onOpenPerpetual(it.id) },
    )
}
