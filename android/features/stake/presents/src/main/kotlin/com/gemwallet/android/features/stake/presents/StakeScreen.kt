package com.gemwallet.android.features.stake.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.stake.viewmodels.StakeViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction

@Composable
fun StakeScreen(amountAction: AmountTransactionAction, onConfirm: ConfirmTransactionAction, onDelegation: (String, String) -> Unit, onCancel: () -> Unit, viewModel: StakeViewModel = hiltViewModel()) {
    val inSync by viewModel.isSync.collectAsStateWithLifecycle()
    val assetInfo by viewModel.assetInfo.collectAsStateWithLifecycle()
    val viewState by viewModel.viewState.collectAsStateWithLifecycle()
    val loadError by viewModel.loadError.collectAsStateWithLifecycle()

    val stakeAssetInfo = assetInfo
    val state = viewState
    if (stakeAssetInfo == null || state == null) {
        LoadingScene(
            title = stringResource(id = R.string.transfer_stake_title),
            onCancel = onCancel,
        )
    } else {
        StakeScene(
            inSync = inSync,
            assetInfo = stakeAssetInfo,
            state = state,
            loadError = loadError,
            amountAction = amountAction,
            onAction = { action ->
                when (action) {
                    StakeAction.Refresh -> viewModel.onRefresh()
                    is StakeAction.Confirm -> onConfirm(ConfirmTransferInput(action.transfer))
                    is StakeAction.OpenDelegation -> viewModel.onDelegation(action.delegation, onDelegation, amountAction, onConfirm)
                    StakeAction.Cancel -> onCancel()
                }
            },
        )
    }
}
