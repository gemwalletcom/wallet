package com.gemwallet.android.features.qr_scanner.presents

import android.Manifest
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyAction
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import com.google.accompanist.permissions.shouldShowRationale
import com.wallet.core.primitives.QRScanType

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun QRScannerScreen(scanType: QRScanType, onCancel: () -> Unit, titleContent: @Composable () -> Unit = { QRScannerTitle() }, onResult: (String) -> Unit) {
    val context = LocalContext.current
    var isPermissionRequested by remember { mutableStateOf(false) }
    val cameraPermission = rememberPermissionState(Manifest.permission.CAMERA) { isPermissionRequested = true }
    val isPermanentlyDenied = isPermissionRequested && !cameraPermission.status.shouldShowRationale
    LaunchedEffect(Unit) {
        if (!cameraPermission.status.isGranted) {
            cameraPermission.launchPermissionRequest()
        }
    }
    BackHandler(true) {
        onCancel()
    }
    QRScannerScene(
        scanType = scanType,
        isCameraGranted = cameraPermission.status.isGranted,
        permissionAction = if (isPermanentlyDenied) {
            EmptyAction(title = stringResource(R.string.common_open_settings), onClick = context::openAppSettings)
        } else {
            EmptyAction(title = stringResource(R.string.common_grant_permission), onClick = cameraPermission::launchPermissionRequest)
        },
        onCancel = onCancel,
        titleContent = titleContent,
        onResult = onResult,
    )
}

private fun Context.openAppSettings() {
    startActivity(Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", packageName, null)))
}
