package com.gemwallet.android.ui.components

import android.Manifest
import android.graphics.Bitmap
import android.graphics.ImageDecoder
import android.graphics.ImageFormat
import android.net.Uri
import android.util.Size
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.StringRes
import androidx.camera.core.ExperimentalGetImage
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import androidx.compose.ui.graphics.ClipOp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.LocalLifecycleOwner
import coil3.compose.AsyncImage
import coil3.request.CachePolicy
import coil3.request.ImageRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space24
import com.wallet.core.primitives.QRScanType
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import com.google.zxing.BarcodeFormat
import com.google.zxing.BinaryBitmap
import com.google.zxing.DecodeHintType
import com.google.zxing.LuminanceSource
import com.google.zxing.MultiFormatReader
import com.google.zxing.PlanarYUVLuminanceSource
import com.google.zxing.RGBLuminanceSource
import com.google.zxing.common.HybridBinarizer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.nio.ByteBuffer
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
    onResult: (String) -> Unit,
) {
    val cameraPermissionState = rememberPermissionState(permission = Manifest.permission.CAMERA)
    var showPermissionRequest by remember { mutableStateOf(true) }

    BackHandler(true) {
        onCancel()
    }

    if (!cameraPermissionState.status.isGranted && showPermissionRequest) {
        AlertDialog(
            onDismissRequest = { showPermissionRequest = false },
            text = {
                Text(text = stringResource(id = R.string.camera_permission_request_camera))
            },
            confirmButton = {
                Button(
                    onClick = {
                        cameraPermissionState.launchPermissionRequest()
                    }
                ) {
                    Text(text = stringResource(id = R.string.common_grant_permission))
                }
            },
            dismissButton = {
                Button(onClick = { showPermissionRequest = false }) {
                    Text(text = stringResource(id = R.string.common_cancel))
                }
            }
        )
    } else {
        QRScannerScene(
            scanType = scanType,
            isCameraGranted = cameraPermissionState.status.isGranted,
            onGrantPermission = { showPermissionRequest = true },
            onCancel = onCancel,
            onResult = onResult
        )
    }
}

@androidx.annotation.OptIn(ExperimentalGetImage::class)
@Composable
fun QRScannerScene(
    scanType: QRScanType,
    isCameraGranted: Boolean,
    onGrantPermission: () -> Unit,
    onCancel: () -> Unit,
    onResult: (String) -> Unit,
) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()
    var imageUri by remember { mutableStateOf<Uri?>(null) }
    var imageResult by remember { mutableStateOf("") }
    var imageError by remember { mutableStateOf("") }
    val galleryLauncher = rememberLauncherForActivityResult(contract = ActivityResultContracts.GetContent()) { uri: Uri? ->
        imageUri = uri
        imageResult = ""
        imageError = ""
    }
    val cancel = {
        imageUri = null
        imageError = ""
        imageResult = ""
    }
    LaunchedEffect(imageUri) {
        val image = imageUri ?: return@LaunchedEffect
        coroutineScope.launch(Dispatchers.IO) {
            try {
                val bitmap = ImageDecoder.decodeBitmap(ImageDecoder.createSource(context.contentResolver, image))
                    .copy(Bitmap.Config.RGBA_F16, true)
                val intArray = IntArray(bitmap.getWidth() * bitmap.getHeight())
                bitmap.getPixels(intArray, 0, bitmap.getWidth(), 0, 0, bitmap.getWidth(), bitmap.getHeight())

                val source = RGBLuminanceSource(
                    bitmap.getWidth(),
                    bitmap.getHeight(),
                    intArray
                )
                val binaryBmp = BinaryBitmap(HybridBinarizer(source))
                val result = MultiFormatReader().apply {
                    setHints(
                        mapOf(DecodeHintType.POSSIBLE_FORMATS to arrayListOf(BarcodeFormat.QR_CODE))
                    )
                }.decode(binaryBmp)
                imageResult = result.text.orEmpty()
                if (imageResult.isBlank()) {
                    throw Exception()
                }
            } catch (e: Exception) {
                imageError = e.message ?: "Unknown error"
            }
        }
    }
    BackHandler(imageUri != null) {
        cancel()
    }
    Scene(
        title = stringResource(id = R.string.wallet_scan),
        actions = {
            if (!isCameraGranted) {
                IconButton(onClick = onGrantPermission) {
                    Icon(imageVector = AppIcons.Camera, contentDescription = "from_camera")
                }
            }
            IconButton(
                onClick = { galleryLauncher.launch("image/*") },
                modifier = Modifier.testTag(SCAN_FROM_GALLERY_TAG),
            ) {
                Icon(imageVector = AppIcons.Image, contentDescription = "from_image")
            }
            if (imageUri != null) {
                IconButton(onClick = cancel) {
                    Icon(imageVector = AppIcons.Close, contentDescription = "close_image")
                }
            }
        },
        onClose = { if (imageUri == null) onCancel() else cancel() },
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            QRScanner(listener = onResult)
            ScannerHint(hint = stringResource(id = scanType.hintRes()))
            if (imageUri != null) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(Color.Black)
                ) {
                    AsyncImage(
                        model = ImageRequest.Builder(LocalContext.current)
                            .data(imageUri)
                            .diskCachePolicy(policy = CachePolicy.ENABLED)
                            .networkCachePolicy(policy = CachePolicy.ENABLED)
                            .build(),
                        contentDescription = "",
                        contentScale = ContentScale.Fit,
                        modifier = Modifier.fillMaxSize(),
                    )
                    if (imageResult.isNotEmpty()) {
                        Column(
                            modifier = Modifier
                                .padding(40.dp)
                                .align(Alignment.BottomCenter),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Text(
                                modifier = Modifier
                                    .defaultPadding()
                                    .background(Color.Black, MaterialTheme.shapes.medium)
                                    .defaultPadding(),
                                text = imageResult,
                                color = Color.White,
                                textAlign = TextAlign.Center,
                                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.W300)
                            )

                            Button(
                                onClick = { onResult(imageResult) }
                            ) {
                                Text(text = stringResource(id = R.string.common_done))
                            }
                        }
                    }
                    if (imageError.isNotEmpty()) {
                        Text(
                            modifier = Modifier
                                .padding(40.dp)
                                .align(Alignment.BottomCenter)
                                .defaultPadding()
                                .background(Color.Black, MaterialTheme.shapes.medium)
                                .defaultPadding(),
                            text = stringResource(id = R.string.errors_decoding_qr),
                            color = Color.White,
                            textAlign = TextAlign.Center,
                            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.W300)
                        )
                    }
                }
            }
        }
    }
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
    LaunchedEffect(Unit) {
        try {
            val provider = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
                androidx.camera.lifecycle.ProcessCameraProvider.getInstance(localContext).get()
            }
            val preview = androidx.camera.core.Preview.Builder()
                .build()
                .also {
                    it.surfaceProvider = previewView.surfaceProvider
                }
            val imageAnalyzer = androidx.camera.core.ImageAnalysis.Builder()
                .setBackpressureStrategy(androidx.camera.core.ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                .setResolutionSelector(
                    androidx.camera.core.resolutionselector.ResolutionSelector.Builder()
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
                        ContextCompat.getMainExecutor(localContext),
                        QRCodeAnalyzer(callback = {
                            imageAnalysis.clearAnalyzer()
                            listener.invoke(it)
                        })
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

@ExperimentalGetImage
private class QRCodeAnalyzer(
    val callback: (String) -> Unit
) : androidx.camera.core.ImageAnalysis.Analyzer {
    private val supportedImageFormats = listOf(
        ImageFormat.YUV_420_888,
        ImageFormat.YUV_422_888,
        ImageFormat.YUV_444_888
    )
    private val reader = MultiFormatReader()
    private val hints = mapOf(
        DecodeHintType.POSSIBLE_FORMATS to listOf(BarcodeFormat.QR_CODE),
        DecodeHintType.TRY_HARDER to true,
    )

    override fun analyze(imageProxy: androidx.camera.core.ImageProxy) {
        if (imageProxy.format !in supportedImageFormats) {
            return
        }
        val bytes = imageProxy.planes.first().buffer.toByteArray()
        val source = PlanarYUVLuminanceSource(
            bytes,
            imageProxy.width,
            imageProxy.height,
            0,
            0,
            imageProxy.width,
            imageProxy.height,
            false
        )
        try {
            val text = tryDecode(source) ?: tryDecode(source.invert())
            text?.let(callback)
        } finally {
            imageProxy.close()
        }
    }

    private fun tryDecode(source: LuminanceSource): String? = try {
        reader.decode(BinaryBitmap(HybridBinarizer(source)), hints).text
    } catch (_: Exception) {
        null
    }
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