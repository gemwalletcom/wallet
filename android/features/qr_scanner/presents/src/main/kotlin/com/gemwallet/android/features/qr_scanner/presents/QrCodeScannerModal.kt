package com.gemwallet.android.features.qr_scanner.presents

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.wallet.core.primitives.QRScanType

@Composable
fun QrCodeScannerModal(isVisible: Boolean, scanType: QRScanType, onDismissRequest: () -> Unit, onResult: (String) -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismissRequest,
        expansion = SheetExpansion.Full,
        shape = RectangleShape,
        dragHandle = null,
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            QrCodeRequest(scanType = scanType, onCancel = onDismissRequest, onResult = onResult)
        }
    }
}
