package com.gemwallet.android.features.perpetual.views.market

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.snapshotFlow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualMarketViewModel

@Composable
fun PerpetualMarketNavScreen(
    onCancel: () -> Unit,
    onOpenPerpetualDetails: (String) -> Unit,
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

    PerpetualMarketScene(
        sceneState = sceneState,
        balance = balance,
        unpinnedPerpetuals = unpinnedPerpetuals,
        pinnedPerpetuals = pinnedPerpetuals,
        positions = positions,
        query = query,
        onRefresh = viewModel::onRefresh,
        onPin = viewModel::onTogglePin,
        onClose = onCancel,
        onWithdraw = {},
        onDeposit = {},
        onClick = onOpenPerpetualDetails
    )
}
