package com.gemwallet.android.ui.components

import android.Manifest
import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.graphics.ImageDecoder
import android.net.Uri
import android.provider.Settings
import android.util.Size
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.StringRes
import androidx.camera.core.ExperimentalGetImage
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.graphics.ClipOp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.buttons.mainActionButtonColors
import com.gemwallet.android.ui.components.empty.EmptyAction
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.SceneTitle
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space24
import com.wallet.core.primitives.QRScanType
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import com.google.accompanist.permissions.shouldShowRationale
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.IOException
import java.nio.ByteBuffer
import java.util.concurrent.Executors
import kotlin.math.min

private val QR_ANALYSIS_RESOLUTION = Size(1280, 720)
private const val SCAN_FROM_GALLERY_TAG = "scanFromGallery"
private const val FINDER_SCALE = 0.66f
private val HINT_SPACING = space24
private val HINT_HORIZONTAL_PADDING = 32.dp
private val FINDER_CORNER_RADIUS = 25.dp
private val FINDER_CORNER_LENGTH = 25.dp
private val FINDER_STROKE_WIDTH = 4.dp
private const val FINDER_DIM_ALPHA = 0.33f

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun QrCodeRequest(
    scanType: QRScanType,
    onCancel: () -> Unit,
    titleContent: @Composable () -> Unit = { ScanQrCodeTitle() },
    onResult: (String) -> Unit,
) {
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

@androidx.annotation.OptIn(ExperimentalGetImage::class)
@Composable
fun QRScannerScene(
    scanType: QRScanType,
    isCameraGranted: Boolean,
    permissionAction: EmptyAction,
    onCancel: () -> Unit,
    titleContent: @Composable () -> Unit = { ScanQrCodeTitle() },
    onResult: (String) -> Unit,
) {
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
        mainAction = if (isCameraGranted) null else {
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
                QRScanner(listener = onResult)
                ScannerHint(hint = stringResource(id = scanType.hintRes()))
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

private fun Context.openAppSettings() {
    startActivity(Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", packageName, null)))
}

private fun ContentResolver.decodeQrCode(uri: Uri): String? = try {
    val bitmap = ImageDecoder.decodeBitmap(ImageDecoder.createSource(this, uri)) { decoder, info, _ ->
        decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
        decoder.setTargetSampleSize(QRCodeDecoder.sampleSize(info.size.width, info.size.height))
    }
    val pixels = IntArray(bitmap.width * bitmap.height)
    bitmap.getPixels(pixels, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
    QRCodeDecoder.decode(pixels, bitmap.width, bitmap.height)
} catch (_: IOException) {
    null
}

@ExperimentalGetImage
@Composable
fun QRScanner(listener: (String) -> Unit) {
    val localContext = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val previewView = remember {
        androidx.camera.view.PreviewView(localContext).also {
            it.scaleType = androidx.camera.view.PreviewView.ScaleType.FILL_CENTER
        }
    }
    val analysisExecutor = remember { Executors.newSingleThreadExecutor() }
    DisposableEffect(Unit) {
        onDispose { analysisExecutor.shutdown() }
    }
    LaunchedEffect(Unit) {
        try {
            val provider = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                androidx.camera.lifecycle.ProcessCameraProvider.getInstance(localContext).get()
            }
            val mainExecutor = ContextCompat.getMainExecutor(localContext)
            val preview = androidx.camera.core.Preview.Builder()
                .build()
                .also {
                    it.surfaceProvider = previewView.surfaceProvider
                }
            val imageAnalyzer = androidx.camera.core.ImageAnalysis.Builder()
                .setBackpressureStrategy(androidx.camera.core.ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                .setResolutionSelector(
                    androidx.camera.core.resolutionselector.ResolutionSelector.Builder()
                        .setAspectRatioStrategy(androidx.camera.core.resolutionselector.AspectRatioStrategy.RATIO_16_9_FALLBACK_AUTO_STRATEGY)
                        .setResolutionStrategy(
                            androidx.camera.core.resolutionselector.ResolutionStrategy(
                                QR_ANALYSIS_RESOLUTION,
                                androidx.camera.core.resolutionselector.ResolutionStrategy.FALLBACK_RULE_CLOSEST_HIGHER_THEN_LOWER,
                            )
                        )
                        .build()
                )
                .build()
                .also { imageAnalysis ->
                    imageAnalysis.setAnalyzer(
                        analysisExecutor,
                        QRCodeAnalyzer { text ->
                            imageAnalysis.clearAnalyzer()
                            mainExecutor.execute { listener(text) }
                        }
                    )
                }
            val selector = androidx.camera.core.CameraSelector.Builder()
                .requireLensFacing(androidx.camera.core.CameraSelector.LENS_FACING_BACK)
                .build()
            provider.unbindAll()
            provider.bindToLifecycle(
                lifecycleOwner,
                selector,
                preview,
                imageAnalyzer,
            )
        } catch (_: Throwable) { }
    }
    Box(modifier = Modifier.fillMaxSize()) {
        AndroidView({ previewView }, modifier = Modifier.fillMaxSize())
        Box(modifier = Modifier
            .fillMaxSize()
        ) {
            FinderView()
        }
    }
}

private class QRCodeAnalyzer(
    val callback: (String) -> Unit
) : androidx.camera.core.ImageAnalysis.Analyzer {

    override fun analyze(imageProxy: androidx.camera.core.ImageProxy) {
        imageProxy.use {
            val plane = it.planes.first()
            QRCodeDecoder.decode(plane.buffer.toByteArray(), plane.rowStride, it.width, it.height)?.let(callback)
        }
    }
}

@Composable
private fun ScanQrCodeTitle() {
    SceneTitle(stringResource(id = R.string.wallet_scan))
}

@StringRes
private fun QRScanType.hintRes(): Int = when (this) {
    QRScanType.Universal -> R.string.wallet_scan_hint
    QRScanType.WalletConnect -> R.string.wallet_connect_title
    QRScanType.Address -> R.string.wallet_scan_hint_address
    QRScanType.Memo -> R.string.transfer_memo
    QRScanType.Url -> R.string.common_url
    QRScanType.TokenContract -> R.string.wallet_import_contract_address_field
    QRScanType.SecretPhrase -> R.string.common_secret_phrase
    QRScanType.PrivateKey -> R.string.common_private_key
}

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

@Composable
private fun FinderView() {
    Canvas(modifier = Modifier.fillMaxSize()) {
        val boxSize = min(size.width, size.height) * FINDER_SCALE
        val left = (size.width - boxSize) / 2f
        val top = (size.height - boxSize) / 2f
        val right = left + boxSize
        val bottom = top + boxSize
        val cornerRadius = FINDER_CORNER_RADIUS.toPx()
        val cornerLength = FINDER_CORNER_LENGTH.toPx()
        val cornerDiameter = cornerRadius * 2f

        val cutout = Path().apply {
            addRoundRect(RoundRect(left, top, right, bottom, CornerRadius(cornerRadius)))
        }
        clipPath(cutout, clipOp = ClipOp.Difference) {
            drawRect(Color.Black.copy(alpha = FINDER_DIM_ALPHA), topLeft = Offset.Zero, size)
        }

        val brackets = Path().apply {
            moveTo(left, top + cornerRadius)
            arcTo(Rect(left, top, left + cornerDiameter, top + cornerDiameter), 180f, 90f, false)
            moveTo(left + cornerRadius, top)
            lineTo(left + cornerRadius + cornerLength, top)
            moveTo(left, top + cornerRadius)
            lineTo(left, top + cornerRadius + cornerLength)

            moveTo(right - cornerRadius, top)
            arcTo(Rect(right - cornerDiameter, top, right, top + cornerDiameter), 270f, 90f, false)
            moveTo(right - cornerRadius - cornerLength, top)
            lineTo(right - cornerRadius, top)
            moveTo(right, top + cornerRadius)
            lineTo(right, top + cornerRadius + cornerLength)

            moveTo(right, bottom - cornerRadius)
            arcTo(Rect(right - cornerDiameter, bottom - cornerDiameter, right, bottom), 0f, 90f, false)
            moveTo(right - cornerRadius - cornerLength, bottom)
            lineTo(right - cornerRadius, bottom)
            moveTo(right, bottom - cornerRadius)
            lineTo(right, bottom - cornerRadius - cornerLength)

            moveTo(left + cornerRadius, bottom)
            arcTo(Rect(left, bottom - cornerDiameter, left + cornerDiameter, bottom), 90f, 90f, false)
            moveTo(left + cornerRadius, bottom)
            lineTo(left + cornerRadius + cornerLength, bottom)
            moveTo(left, bottom - cornerRadius)
            lineTo(left, bottom - cornerRadius - cornerLength)
        }
        drawPath(
            path = brackets,
            color = Color.White,
            style = Stroke(width = FINDER_STROKE_WIDTH.toPx(), cap = StrokeCap.Round, join = StrokeJoin.Round),
        )
    }
}

private fun ByteBuffer.toByteArray(): ByteArray {
    rewind()
    return ByteArray(remaining()).also {
        get(it)
    }
}