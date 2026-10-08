package com.gemwallet.android.ui.components

import android.os.SystemClock
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.foundation.text.TextAutoSize
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

val LocalNumericTransition = staticCompositionLocalOf { false }

@Composable
fun NumericText(
    text: String,
    modifier: Modifier = Modifier,
    color: Color = Color.Unspecified,
    style: TextStyle = LocalTextStyle.current,
    textAlign: TextAlign? = null,
    overflow: TextOverflow = TextOverflow.Clip,
    maxLines: Int = Int.MAX_VALUE,
    autoSize: TextAutoSize? = null,
    animated: Boolean = true,
) {
    val transition = remember { NumericTransition() }
    val scope = rememberCoroutineScope()
    Text(
        text = text,
        modifier = if (animated) modifier.drawWithContent { if (!transition.draw(this)) drawContent() } else modifier,
        color = color,
        style = style,
        textAlign = textAlign,
        overflow = overflow,
        maxLines = maxLines,
        autoSize = autoSize,
        onTextLayout = if (animated) {
            { transition.update(it, scope) }
        } else {
            null
        },
    )
}

private class NumericTransition {
    private var progress by mutableFloatStateOf(1f)
    private var from: TextLayoutResult? = null
    private var to: TextLayoutResult? = null
    private var job: Job? = null
    private var changedAt = 0L

    fun update(layout: TextLayoutResult, scope: CoroutineScope) {
        val previous = to
        to = layout
        if (previous?.layoutInput?.text == layout.layoutInput.text) return
        val now = SystemClock.uptimeMillis()
        val wasSteady = now - changedAt > STEADY_MILLIS
        changedAt = now
        job?.cancel()
        from = previous?.takeIf { wasSteady && it.isNumeric() && layout.isNumeric() }
        progress = if (from == null) 1f else 0f
        if (from != null) {
            job = scope.launch { animate(0f, 1f, animationSpec = animationSpec) { value, _ -> progress = value } }
        }
    }

    fun draw(scope: DrawScope): Boolean {
        val from = from ?: return false
        val to = to ?: return false
        val fraction = progress
        if (fraction >= 1f) return false
        val old = from.layoutInput.text.text
        val new = to.layoutInput.text.text
        val shift = scope.size.height * SLIDE_FRACTION
        val recolors = from.layoutInput.style.color != to.layoutInput.style.color
        new.indices.forEach { index ->
            val box = to.getBoundingBox(index)
            val oldIndex = index - new.length + old.length
            if (oldIndex >= 0 && old[oldIndex] == new[index]) {
                if (recolors) scope.glyph(from, oldIndex, box, 0f, 1f)
                scope.glyph(to, index, box, 0f, if (recolors) fraction else 1f)
            } else {
                if (oldIndex >= 0) scope.glyph(from, oldIndex, box, shift * fraction, 1f - fraction)
                scope.glyph(to, index, box, -shift * (1f - fraction), fraction)
            }
        }
        return true
    }

    private fun DrawScope.glyph(source: TextLayoutResult, index: Int, box: Rect, dy: Float, alpha: Float) {
        clipRect(left = box.left, top = 0f, right = box.right, bottom = size.height) {
            drawText(source, topLeft = Offset(box.left - source.getBoundingBox(index).left, dy), alpha = alpha)
        }
    }

    private fun TextLayoutResult.isNumeric(): Boolean = layoutInput.text.text.any(Char::isDigit) && !isLineEllipsized(lineCount - 1)

    private companion object {
        const val SLIDE_FRACTION = 0.6f
        const val STEADY_MILLIS = 500L
        val animationSpec = tween<Float>(durationMillis = 300)
    }
}
