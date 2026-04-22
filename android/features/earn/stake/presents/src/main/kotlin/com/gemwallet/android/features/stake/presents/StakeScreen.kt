package com.gemwallet.android.features.stake.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.features.stake.viewmodels.StakeViewModel

@Composable
fun StakeScreen(
    amountAction: AmountTransactionAction,
    onConfirm: ConfirmTransactionAction,
    onDelegation: (String, String) -> Unit,
    onCancel: () -> Unit,
    viewModel: StakeViewModel = hiltViewModel()
) {
    val inSync by viewModel.isSync.collectAsStateWithLifecycle()
    val assetInfo by viewModel.assetInfo.collectAsStateWithLifecycle()
    val delegations by viewModel.delegations.collectAsStateWithLifecycle()
    val isStakeEnabled by viewModel.isStakeEnabled.collectAsStateWithLifecycle()
    val actions by viewModel.actions.collectAsStateWithLifecycle()
    val stakeInfoUrl by viewModel.stakeInfoUrl.collectAsStateWithLifecycle()

    if (assetInfo == null) {
        LoadingScene(
            title = stringResource(id = R.string.transfer_stake_title),
            onCancel = onCancel,
        )
    } else {
        StakeScene(
            inSync = inSync,
            assetInfo = assetInfo!!,
            delegations = delegations,
            actions = actions,
            isStakeEnabled = isStakeEnabled,
            stakeInfoUrl = stakeInfoUrl,
            onRefresh = viewModel::onRefresh,
            amountAction = amountAction,
            onRewards = { viewModel.onRewards(amountAction, onConfirm) },
            onDelegation = { delegation -> viewModel.onDelegation(delegation, onDelegation, onConfirm) },
            onCancel = onCancel
        )
    }
}
