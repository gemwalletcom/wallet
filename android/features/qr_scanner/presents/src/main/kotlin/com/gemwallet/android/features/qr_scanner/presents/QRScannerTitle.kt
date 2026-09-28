package com.gemwallet.android.features.qr_scanner.presents

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.SceneTitle

@Composable
internal fun QRScannerTitle() {
    SceneTitle(stringResource(id = R.string.wallet_scan))
}
