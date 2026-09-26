package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.nft.viewmodels.CollectibleViewModel
import com.gemwallet.android.features.nft.viewmodels.models.ReportReasonUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.models.actions.CancelAction
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.ReportReason
import uniffi.gemstone.GemCollectibleAction

@Composable
fun CollectibleScreen(cancelAction: CancelAction, onRecipient: (NFTAsset) -> Unit, onOpenAddress: (ChainAddress) -> Unit) {
    val viewModel: CollectibleViewModel = hiltViewModel()
    val assetData by viewModel.nftAsset.collectAsStateWithLifecycle()

    val snackbar = remember { SnackbarHostState() }
    ToastEffect(viewModel.toastEvents, snackbar)

    val data = assetData ?: return
    var isReportVisible by remember { mutableStateOf(false) }
    CollectibleScene(
        data = data,
        snackbar = snackbar,
        onClose = { cancelAction() },
        onSend = { onRecipient(data.asset) },
        onAction = { action ->
            when (action) {
                GemCollectibleAction.REFRESH -> viewModel.refresh()
                GemCollectibleAction.SAVE_IMAGE -> viewModel.saveImage()
                GemCollectibleAction.SET_AVATAR -> viewModel.setAsAvatar()
                GemCollectibleAction.REPORT -> isReportVisible = true
            }
        },
        onOpenAddress = onOpenAddress,
    )
    ReportReasonSheet(
        isVisible = isReportVisible,
        reasons = viewModel.reportReasons,
        onDismiss = { isReportVisible = false },
        onSelect = { reason -> viewModel.report(reason) },
    )
}

@Composable
private fun ReportReasonSheet(isVisible: Boolean, reasons: List<ReportReasonUIModel>, onDismiss: () -> Unit, onSelect: (ReportReason) -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.nft_report_report_button_title),
    ) {
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            itemsPositioned(reasons) { position, item ->
                ListItem(
                    model = item.model,
                    listPosition = position,
                    modifier = Modifier.clickable {
                        onSelect(item.reason)
                        onDismiss()
                    },
                    minHeight = ListItemDefaults.plainMinHeight,
                )
            }
        }
    }
}
