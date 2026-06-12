package com.gemwallet.android.features.perpetual.views.market

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.snapshotFlow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.features.asset_select.presents.views.RecentsSheetHost
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualMarketViewModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.AssetIdAction

@Composable
fun PerpetualMarketNavScreen(
    onCancel: () -> Unit,
    onOpenPerpetualDetails: AssetIdAction,
    amountAction: AmountTransactionAction,
    viewModel: PerpetualMarketViewModel = hiltViewModel(),
    recentsViewModel: RecentsSheetViewModel = hiltViewModel(),
) {
    val sceneState by viewModel.sceneState.collectAsStateWithLifecycle()
    val unpinnedPerpetuals by viewModel.unpinnedPerpetuals.collectAsStateWithLifecycle()
    val pinnedPerpetuals by viewModel.pinnedPerpetuals.collectAsStateWithLifecycle()
    val positions by viewModel.positions.collectAsStateWithLifecycle()
    val balance by viewModel.balance.collectAsStateWithLifecycle()
    val recent by viewModel.recent.collectAsStateWithLifecycle()
    val query = rememberTextFieldState()

    LaunchedEffect(query) {
        snapshotFlow { query.text.toString() }.collect(viewModel::setQuery)
    }

    LaunchedEffect(Unit) { viewModel.fetch() }

    PerpetualMarketScene(
        sceneState = sceneState,
        balance = balance,
        unpinnedPerpetuals = unpinnedPerpetuals,
        pinnedPerpetuals = pinnedPerpetuals,
        positions = positions,
        recent = recent,
        query = query,
        onAction = { action ->
            when (action) {
                PerpetualMarketAction.Refresh -> viewModel.onRefresh()
                PerpetualMarketAction.Close -> onCancel()
                PerpetualMarketAction.Withdraw -> amountAction(AmountParams.Withdraw(HypercoreUSDC.id))
                PerpetualMarketAction.Deposit -> amountAction(AmountParams.Deposit(PerpetualConfig.depositAssetId))
                is PerpetualMarketAction.TogglePin -> viewModel.onTogglePin(action.perpetualId)
                is PerpetualMarketAction.OpenPerpetual -> {
                    onOpenPerpetualDetails(action.assetId)
                    viewModel.onOpenPerpetual(action.assetId)
                }
                is PerpetualMarketAction.OpenRecent -> onOpenPerpetualDetails(action.assetId)
                PerpetualMarketAction.OpenRecentsSheet -> recentsViewModel.show(types = listOf(RecentType.Perpetual))
            }
        },
    )

    RecentsSheetHost(
        viewModel = recentsViewModel,
        onSelect = { onOpenPerpetualDetails(it) },
    )
}
