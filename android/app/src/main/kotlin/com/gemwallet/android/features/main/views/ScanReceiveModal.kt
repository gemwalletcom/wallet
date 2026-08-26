package com.gemwallet.android.features.main.views

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.presents.views.SelectReceiveScreen
import com.gemwallet.android.features.receive.presents.ReceiveScreen
import com.gemwallet.android.ui.components.QrCodeRequest
import com.gemwallet.android.ui.components.ScanReceiveMode
import com.gemwallet.android.ui.components.ScanReceiveSwitcher
import com.gemwallet.android.ui.components.screen.ModalBottomSheet

private const val RECEIVE_SHEET_HEIGHT = 0.93f

@Composable
fun ScanReceiveModal(
    isVisible: Boolean,
    onDismissRequest: () -> Unit,
    onScan: (String) -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismissRequest,
        skipPartiallyExpanded = true,
        shape = RectangleShape,
        dragHandle = null,
    ) {
        var mode by rememberSaveable { mutableStateOf(ScanReceiveMode.Scan) }
        var receiveAssetId by rememberSaveable { mutableStateOf<String?>(null) }
        var isReceivePresented by rememberSaveable { mutableStateOf(false) }

        Box(modifier = Modifier.fillMaxSize()) {
            when (mode) {
                ScanReceiveMode.Scan -> QrCodeRequest(
                    onCancel = onDismissRequest,
                    titleContent = { ScanReceiveSwitcher(mode = mode, onModeChange = { mode = it }) },
                    onResult = onScan,
                )
                ScanReceiveMode.Receive -> SelectReceiveScreen(
                    onCancel = onDismissRequest,
                    onSelect = {
                        receiveAssetId = it.toIdentifier()
                        isReceivePresented = true
                    },
                    titleContent = { ScanReceiveSwitcher(mode = mode, onModeChange = { mode = it }) },
                    closeIcon = true,
                )
            }
        }

        ModalBottomSheet(
            isVisible = isReceivePresented,
            onDismissRequest = { isReceivePresented = false },
            skipPartiallyExpanded = true,
        ) {
            receiveAssetId?.toAssetId()?.let { assetId ->
                Box(modifier = Modifier.fillMaxHeight(RECEIVE_SHEET_HEIGHT)) {
                    ReceiveScreen(assetId = assetId, closeIcon = true, onCancel = { isReceivePresented = false })
                }
            }
        }
    }
}
