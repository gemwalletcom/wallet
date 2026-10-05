package com.gemwallet.android.features.stake.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.stake.viewmodels.StakeViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction

@Composable
fun StakeScreen(amountAction: AmountTransactionAction, onConfirm: ConfirmTransactionAction, onDelegation: (String, String) -> Unit, onCancel: () -> Unit, viewModel: StakeViewModel = hiltViewModel()) {
    val infoSheet by viewModel.infoSheet.collectAsStateWithLifecycle()
    InfoBottomSheet(item = infoSheet?.infoSheet(), onClose = { viewModel.infoSheet.value = null })
    val inSync by viewModel.isSync.collectAsStateWithLifecycle()
    val assetInfo by viewModel.assetInfo.collectAsStateWithLifecycle()
    val viewState by viewModel.viewState.collectAsStateWithLifecycle()

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
            onAction = { action ->
                when (action) {
                    StakeAction.Refresh -> viewModel.onRefresh()
                    is StakeAction.Open -> viewModel.onSelect(action.kind, action.destination, amountAction, onConfirm)
                    is StakeAction.OpenDelegation -> viewModel.onDelegation(action.delegation, onDelegation, amountAction, onConfirm)
                    StakeAction.Cancel -> onCancel()
                }
            },
        )
    }
}
