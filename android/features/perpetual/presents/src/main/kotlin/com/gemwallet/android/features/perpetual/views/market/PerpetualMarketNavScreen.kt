package com.gemwallet.android.features.perpetual.views.market

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.snapshotFlow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualMarketViewModel
import com.gemwallet.android.ui.models.actions.AssetIdAction

@Composable
fun PerpetualMarketNavScreen(
    onCancel: () -> Unit,
    onOpenPerpetualDetails: AssetIdAction,
    viewModel: PerpetualMarketViewModel = hiltViewModel(),
) {
    val sceneState by viewModel.sceneState.collectAsStateWithLifecycle()
    val unpinnedPerpetuals by viewModel.unpinnedPerpetuals.collectAsStateWithLifecycle()
    val pinnedPerpetuals by viewModel.pinnedPerpetuals.collectAsStateWithLifecycle()
    val positions by viewModel.positions.collectAsStateWithLifecycle()
    val balance by viewModel.balance.collectAsStateWithLifecycle()
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
        query = query,
        onAction = { action ->
            when (action) {
                PerpetualMarketAction.Refresh -> viewModel.onRefresh()
                PerpetualMarketAction.Close -> onCancel()
                PerpetualMarketAction.Withdraw -> Unit
                PerpetualMarketAction.Deposit -> Unit
                is PerpetualMarketAction.TogglePin -> viewModel.onTogglePin(action.perpetualId)
                is PerpetualMarketAction.OpenPerpetual -> onOpenPerpetualDetails(action.assetId)
            }
        },
    )
}
