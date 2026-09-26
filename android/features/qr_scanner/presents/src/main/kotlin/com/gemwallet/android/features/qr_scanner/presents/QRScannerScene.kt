package com.gemwallet.android.features.qr_scanner.presents

import android.content.ContentResolver
import android.graphics.ImageDecoder
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.StringRes
import androidx.camera.core.ExperimentalGetImage
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.buttons.mainActionButtonColors
import com.gemwallet.android.ui.components.empty.EmptyAction
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space24
import com.wallet.core.primitives.QRScanType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.IOException

private const val SCAN_FROM_GALLERY_TAG = "scanFromGallery"
private val HINT_SPACING = space24
private val HINT_HORIZONTAL_PADDING = paddingLarge

@androidx.annotation.OptIn(ExperimentalGetImage::class)
@Composable
fun QRScannerScene(scanType: QRScanType, isCameraGranted: Boolean, permissionAction: EmptyAction, onCancel: () -> Unit, titleContent: @Composable () -> Unit = { QRScannerTitle() }, onResult: (String) -> Unit) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()
    val haptic = LocalHapticFeedback.current
    val decodingError = stringResource(id = R.string.errors_decoding_qr)
    var toastMessage by remember { mutableStateOf<String?>(null) }
    val snackbar = rememberSnackbarState(
        message = toastMessage,
        iconRes = R.drawable.ic_error,
        onShown = { toastMessage = null },
    )
    val galleryLauncher = rememberLauncherForActivityResult(contract = ActivityResultContracts.GetContent()) { uri: Uri? ->
        val image = uri ?: return@rememberLauncherForActivityResult
        coroutineScope.launch {
            val code = withContext(Dispatchers.IO) { context.contentResolver.decodeQrCode(image) }
            if (code == null) {
                haptic.performHapticFeedback(HapticFeedbackType.Reject)
                toastMessage = decodingError
            } else {
                onResult(code)
            }
        }
    }
    Scene(
        closeIcon = true,
        titleContent = titleContent,
        actions = {
            IconButton(
                onClick = { galleryLauncher.launch("image/*") },
                modifier = Modifier.testTag(SCAN_FROM_GALLERY_TAG),
            ) {
                Icon(imageVector = AppIcons.Image, contentDescription = "from_image")
            }
        },
        onClose = onCancel,
        mainAction = if (isCameraGranted) {
            null
        } else {
            {
                Column(verticalArrangement = Arrangement.spacedBy(paddingSmall)) {
                    MainActionButton(title = permissionAction.title, onClick = permissionAction.onClick)
                    MainActionButton(
                        title = stringResource(id = R.string.library_select_from_photo_library),
                        colors = mainActionButtonColors(
                            containerColor = MaterialTheme.colorScheme.surfaceContainerHighest,
                            contentColor = MaterialTheme.colorScheme.onSurface,
                        ),
                        onClick = { galleryLauncher.launch("image/*") },
                    )
                }
            }
        },
        snackbar = snackbar,
    ) {
        if (isCameraGranted) {
            Box(modifier = Modifier.fillMaxSize()) {
                QRScannerView(listener = onResult)
                ScannerHint(hint = stringResource(id = scanType.stringRes()))
            }
        } else {
            EmptyStateView(
                title = stringResource(id = R.string.errors_permissions_not_granted),
                description = stringResource(id = R.string.errors_camera_permissions_not_granted),
                iconVector = AppIcons.Camera,
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

private fun ContentResolver.decodeQrCode(uri: Uri): String? = try {
    val bitmap = ImageDecoder.decodeBitmap(ImageDecoder.createSource(this, uri)) { decoder, info, _ ->
        decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
        decoder.setTargetSampleSize(QRImageDecoder.sampleSize(info.size.width, info.size.height))
    }
    val pixels = IntArray(bitmap.width * bitmap.height)
    bitmap.getPixels(pixels, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
    QRImageDecoder.decode(pixels, bitmap.width, bitmap.height)
} catch (_: IOException) {
    null
}

@StringRes
@Composable
private fun ScannerHint(hint: String) {
    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val frameSize = minOf(maxWidth, maxHeight)
        val topInset = maxHeight / 2 + frameSize * FINDER_SCALE / 2 + HINT_SPACING
        Text(
            text = hint,
            color = Color.White,
            textAlign = TextAlign.Center,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier
                .padding(top = topInset)
                .align(Alignment.TopCenter)
                .padding(horizontal = HINT_HORIZONTAL_PADDING)
                .background(Color.Black.copy(alpha = alpha50), RoundedCornerShape(percent = 50))
                .padding(horizontal = padding16, vertical = paddingSmall),
        )
    }
}
