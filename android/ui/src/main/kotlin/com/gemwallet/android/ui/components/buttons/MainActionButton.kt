package com.gemwallet.android.ui.components.buttons

import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.mainActionHeight

@Composable
fun MainActionButton(
    title: String,
    modifier: Modifier = Modifier,
    state: ButtonState = ButtonState.Enabled,
    colors: ButtonColors = mainActionButtonColors(),
    onClick: () -> Unit,
) {
    MainActionButton(modifier, state, colors, onClick) {
        if (state == ButtonState.Loading) {
            CircularProgressIndicator20(color = colors.contentColor)
        } else {
            Text(
                modifier = Modifier.padding(4.dp),
                text = title,
                fontSize = 18.sp,
                textAlign = TextAlign.Center,
            )
        }
    }
}

@Composable
fun MainActionButton(
    modifier: Modifier = Modifier,
    state: ButtonState = ButtonState.Enabled,
    colors: ButtonColors = mainActionButtonColors(),
    onClick: () -> Unit,
    content: @Composable RowScope.() -> Unit
) {
    Button(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = mainActionHeight)
            .testTag("main_action"),
        onClick = onClick,
        enabled = state == ButtonState.Enabled,
        colors = colors.forState(state),
    ) {
        content()
    }
}

private fun ButtonColors.forState(state: ButtonState): ButtonColors = when (state) {
    ButtonState.Loading -> copy(
        disabledContainerColor = containerColor,
        disabledContentColor = contentColor,
    )
    else -> this
}

@Composable
fun mainActionButtonColors(
    containerColor: Color = MaterialTheme.colorScheme.primary,
    contentColor: Color = MaterialTheme.colorScheme.onPrimary,
): ButtonColors = ButtonDefaults.buttonColors(
    containerColor = containerColor,
    contentColor = contentColor,
    disabledContainerColor = containerColor.copy(alpha = alpha50),
    disabledContentColor = contentColor,
)

@Composable
fun secondaryActionButtonColors(): ButtonColors = mainActionButtonColors(
    containerColor = MaterialTheme.colorScheme.surface,
    contentColor = MaterialTheme.colorScheme.onSurface,
)
