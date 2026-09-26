package com.gemwallet.android.features.transfer.presents.confirm.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.AcquireAssetRequest
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.ConfirmErrorUIModel
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.GetAssetAction
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.GetAssetOptionUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.AssetId

@Composable
internal fun ConfirmErrorInfo(
    error: ConfirmErrorUIModel?,
    acquireRequest: AcquireAssetRequest?,
    acquireOptions: List<GetAssetOptionUIModel>,
    isShowBottomSheetInfo: Boolean,
    onDismissBottomSheetInfo: () -> Unit,
    onDismissAcquire: () -> Unit,
    onGetAsset: (GetAssetAction, AssetId) -> Unit,
) {
    var isShowInfoSheet by remember { mutableStateOf(false) }

    LaunchedEffect(acquireRequest) {
        val request = acquireRequest ?: return@LaunchedEffect
        isShowInfoSheet = false
        onDismissBottomSheetInfo()
        if (!request.offersOptions) {
            onDismissAcquire()
            onGetAsset(GetAssetAction.Buy(request.buyAmount), request.asset.id)
        }
    }

    GetAssetSheet(
        asset = acquireRequest?.takeIf { it.offersOptions }?.asset,
        options = acquireOptions,
        onDismiss = onDismissAcquire,
        onAction = { action ->
            onDismissAcquire()
            acquireRequest?.let { onGetAsset(action, it.asset.id) }
        },
    )

    error ?: return

    WarningItem(
        title = stringResource(R.string.errors_error_occurred),
        message = error.text,
        color = MaterialTheme.colorScheme.error,
        position = ListPosition.Single,
        onClick = error.info?.let { { isShowInfoSheet = true } },
    )

    if (isShowInfoSheet || isShowBottomSheetInfo) {
        InfoBottomSheet(item = error.info) {
            isShowInfoSheet = false
            onDismissBottomSheetInfo()
        }
    }
}
