package com.gemwallet.android.features.create_wallet.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space12

private enum class WordState {
    Error,
    Done,
    Idle,
}

@Composable
internal fun WordChip(
    word: String,
    isEnable: Boolean,
    onClick: (String) -> Boolean,
) {
    val shakeController = rememberShakeController()
    var wordState by remember { mutableStateOf(WordState.Idle) }
    val shape = RoundedCornerShape(space12)
    val bgColor: Color by animateColorAsState(
        when {
            wordState == WordState.Error -> MaterialTheme.colorScheme.error
            !isEnable -> MaterialTheme.colorScheme.surfaceVariant
            else -> MaterialTheme.colorScheme.primary
        }, label = "Button color"
    )
    val textColor = if (isEnable) Color.White else MaterialTheme.colorScheme.onSurfaceVariant
    Surface(
        modifier = Modifier
            .shake(shakeController, onComplete = { wordState = WordState.Idle })
            .clip(shape)
            .clickable(enabled = wordState != WordState.Idle || isEnable) {
                if (!isEnable) {
                    return@clickable
                }
                if (!onClick(word)) {
                    shakeController.shake(
                        ShakeConfig(
                            iterations = 2,
                            intensity = 1_000f,
                            rotateY = 10f,
                            translateX = 10f,
                        )
                    )
                    wordState = WordState.Error
                }
            },
        shape = shape,
        color = bgColor,
    ) {
        Text(
            text = word,
            color = textColor,
            modifier = Modifier.padding(horizontal = paddingSmall, vertical = paddingHalfSmall),
        )
    }
}
