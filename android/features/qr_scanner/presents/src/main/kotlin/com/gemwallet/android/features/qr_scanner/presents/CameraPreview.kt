package com.gemwallet.android.features.qr_scanner.presents

import android.content.Context
import android.graphics.SurfaceTexture
import android.hardware.display.DisplayManager
import android.view.Surface
import android.view.TextureView
import androidx.camera.core.Preview
import androidx.camera.core.SurfaceRequest
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import java.util.concurrent.Executor
import kotlin.math.max
import kotlin.math.roundToInt

@Composable
internal fun CameraPreview(preview: Preview, modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val texture = remember { PreviewTexture(context, ContextCompat.getMainExecutor(context)) }
    var transformation by remember { mutableStateOf<SurfaceRequest.TransformationInfo?>(null) }
    val displayRotation = rememberDisplayRotation()
    DisposableEffect(preview) {
        preview.setSurfaceProvider(texture.executor) { request ->
            request.setTransformationInfoListener(texture.executor) { transformation = it }
            texture.provide(request)
        }
        onDispose { preview.setSurfaceProvider(null) }
    }
    Box(modifier = modifier.clipToBounds()) {
        AndroidView(factory = { texture.view }, modifier = transformation?.let { Modifier.cover(it, displayRotation) } ?: Modifier.fillMaxSize())
    }
}

@Composable
private fun rememberDisplayRotation(): Int {
    val view = LocalView.current
    var rotation by remember { mutableIntStateOf(view.display?.rotation ?: Surface.ROTATION_0) }
    DisposableEffect(view) {
        val displays = view.context.getSystemService(DisplayManager::class.java)
        val listener = object : DisplayManager.DisplayListener {
            override fun onDisplayChanged(displayId: Int) {
                view.display?.takeIf { it.displayId == displayId }?.let { rotation = it.rotation }
            }

            override fun onDisplayAdded(displayId: Int) = Unit

            override fun onDisplayRemoved(displayId: Int) = Unit
        }
        displays.registerDisplayListener(listener, null)
        onDispose { displays.unregisterDisplayListener(listener) }
    }
    return rotation
}

private fun Modifier.cover(transformation: SurfaceRequest.TransformationInfo, displayRotation: Int): Modifier {
    val rotation = if (transformation.hasCameraTransform()) -surfaceRotationDegrees(displayRotation) else transformation.rotationDegrees
    val crop = transformation.cropRect
    val turned = transformation.rotationDegrees % 180 != 0
    val viewportWidth = if (turned) crop.height() else crop.width()
    val viewportHeight = if (turned) crop.width() else crop.height()
    val swapped = rotation % 180 != 0
    return layout { measurable, constraints ->
        val scale = max(constraints.maxWidth.toFloat() / viewportWidth, constraints.maxHeight.toFloat() / viewportHeight)
        val width = (viewportWidth * scale).roundToInt()
        val height = (viewportHeight * scale).roundToInt()
        val placeable = measurable.measure(Constraints.fixed(if (swapped) height else width, if (swapped) width else height))
        layout(constraints.maxWidth, constraints.maxHeight) {
            placeable.place((constraints.maxWidth - placeable.width) / 2, (constraints.maxHeight - placeable.height) / 2)
        }
    }.graphicsLayer { rotationZ = rotation.toFloat() }
}

private fun surfaceRotationDegrees(surfaceRotation: Int): Int = when (surfaceRotation) {
    Surface.ROTATION_90 -> 90
    Surface.ROTATION_180 -> 180
    Surface.ROTATION_270 -> 270
    else -> 0
}

private class PreviewTexture(context: Context, val executor: Executor) : TextureView.SurfaceTextureListener {
    val view = TextureView(context).also { it.surfaceTextureListener = this }
    private var pending: SurfaceRequest? = null
    private var surfacesInUse = 0
    private var detached: SurfaceTexture? = null

    fun provide(request: SurfaceRequest) {
        pending?.willNotProvideSurface()
        pending = request
        view.surfaceTexture?.let(::attach)
    }

    private fun attach(texture: SurfaceTexture) {
        val request = pending ?: return
        pending = null
        texture.setDefaultBufferSize(request.resolution.width, request.resolution.height)
        val surface = Surface(texture)
        surfacesInUse += 1
        request.provideSurface(surface, executor) {
            surface.release()
            surfacesInUse -= 1
            if (surfacesInUse == 0) {
                detached?.release()
                detached = null
            }
        }
    }

    override fun onSurfaceTextureAvailable(texture: SurfaceTexture, width: Int, height: Int) = attach(texture)

    override fun onSurfaceTextureSizeChanged(texture: SurfaceTexture, width: Int, height: Int) = Unit

    override fun onSurfaceTextureUpdated(texture: SurfaceTexture) = Unit

    override fun onSurfaceTextureDestroyed(texture: SurfaceTexture): Boolean {
        if (surfacesInUse == 0) return true
        detached = texture
        return false
    }
}
