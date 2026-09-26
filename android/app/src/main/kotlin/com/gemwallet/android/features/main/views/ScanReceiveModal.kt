package com.gemwallet.android.features.main.views

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.assets.presents.select.SelectReceiveScreen
import com.gemwallet.android.features.assets.presents.select.assetSelectViewModel
import com.gemwallet.android.features.receive.presents.ReceiveScreen
import com.gemwallet.android.ui.components.PortraitOrientationLock
import com.gemwallet.android.ui.components.QrCodeRequest
import com.gemwallet.android.ui.components.ScanReceiveSwitcher
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.theme.SheetSizing
import com.wallet.core.primitives.QRScanType
import com.wallet.core.primitives.ScanReceiveMode
import uniffi.gemstone.GemSelectAssetType

@Composable
fun ScanReceiveModal(isVisible: Boolean, onDismissRequest: () -> Unit, onScan: (String) -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismissRequest,
        expansion = SheetExpansion.Full,
        shape = RectangleShape,
        dragHandle = null,
    ) {
        PortraitOrientationLock()

        val assetSelectViewModel = assetSelectViewModel(GemSelectAssetType.Receive)
        DisposableEffect(Unit) {
            onDispose { assetSelectViewModel.reset() }
        }

        var mode by rememberSaveable { mutableStateOf(ScanReceiveMode.Scan) }
        var receiveAssetId by rememberSaveable { mutableStateOf<String?>(null) }
        var isReceivePresented by rememberSaveable { mutableStateOf(false) }

        Box(modifier = Modifier.fillMaxSize()) {
            when (mode) {
                ScanReceiveMode.Scan -> QrCodeRequest(
                    scanType = QRScanType.Universal,
                    onCancel = onDismissRequest,
                    titleContent = { ScanReceiveSwitcher(mode = mode, onModeChange = { mode = it }) },
                    onResult = onScan,
                )

                ScanReceiveMode.Receive -> SelectReceiveScreen(
                    viewModel = assetSelectViewModel,
                    onCancel = onDismissRequest,
                    onSelect = {
                        receiveAssetId = it.toIdentifier()
                        isReceivePresented = true
                    },
                    titleContent = { ScanReceiveSwitcher(mode = mode, onModeChange = { mode = it }) },
                    closeIcon = true,
                    showFilter = false,
                )
            }
        }

        ModalBottomSheet(
            isVisible = isReceivePresented,
            onDismissRequest = { isReceivePresented = false },
            expansion = SheetExpansion.Full,
        ) {
            receiveAssetId?.toAssetId()?.let { assetId ->
                Box(modifier = Modifier.fillMaxHeight(SheetSizing.heightFraction)) {
                    ReceiveScreen(assetId = assetId, closeIcon = true, onCancel = { isReceivePresented = false })
                }
            }
        }
    }
}
