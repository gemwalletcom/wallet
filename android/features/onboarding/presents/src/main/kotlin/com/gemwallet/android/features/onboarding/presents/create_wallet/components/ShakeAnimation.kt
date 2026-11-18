package com.gemwallet.android.features.onboarding.presents.create_wallet.components

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.spring
import androidx.compose.foundation.layout.offset
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.IntOffset
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

data class ShakeConfig(
    val iterations: Int,
    val intensity: Float = 100_000f,
    val rotate: Float = 0f,
    val rotateX: Float = 0f,
    val rotateY: Float = 0f,
    val scaleX: Float = 0f,
    val scaleY: Float = 0f,
    val translateX: Float = 0f,
    val translateY: Float = 0f,
)

@Composable
fun Modifier.shake(shakeController: ShakeController) = composed {
    shakeController.shakeConfig?.let { shakeConfig ->
        val shake = shakeController.offset

        this.rotate(shake.value * shakeConfig.rotate)
            .graphicsLayer {
                rotationX = shake.value * shakeConfig.rotateX
                rotationY = shake.value * shakeConfig.rotateY
            }
            .scale(
                scaleX = 1f + (shake.value * shakeConfig.scaleX),
                scaleY = 1f + (shake.value * shakeConfig.scaleY),
            )
            .offset {
                IntOffset(
                    (shake.value * shakeConfig.translateX).roundToInt(),
                    (shake.value * shakeConfig.translateY).roundToInt(),
                )
            }
    } ?: this
}

@Composable
fun rememberShakeController(): ShakeController {
    val scope = rememberCoroutineScope()
    return remember(scope) { ShakeController(scope) }
}

class ShakeController(private val scope: CoroutineScope) {
    val offset = Animatable(0f)
    var shakeConfig: ShakeConfig? by mutableStateOf(null)
        private set

    fun shake(shakeConfig: ShakeConfig, onComplete: () -> Unit) {
        this.shakeConfig = shakeConfig
        scope.launch {
            for (i in 0..shakeConfig.iterations) {
                offset.animateTo(if (i % 2 == 0) 1f else -1f, spring(stiffness = shakeConfig.intensity))
            }
            offset.animateTo(0f)
            onComplete()
        }
    }
}
