package com.gemwallet.android.features.transfer.presents.components

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.isKeyboardVisible
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.theme.SceneSizing

@Composable
internal fun ContinueScene(title: String, buttonState: ButtonState, onClose: () -> Unit, onContinue: () -> Unit, content: @Composable ColumnScope.(PaddingValues) -> Unit) {
    val isKeyBoardOpen = WindowInsets.isKeyboardVisible
    val density = LocalDensity.current
    val isSmallScreen = with(density) {
        LocalWindowInfo.current.containerSize.height.toDp() < SceneSizing.compactContentHeight
    }

    Scene(
        title = title,
        onClose = onClose,
        mainAction = {
            if (!isKeyBoardOpen || !isSmallScreen) {
                MainActionButton(
                    title = stringResource(id = R.string.common_continue),
                    state = buttonState,
                    onClick = onContinue,
                )
            }
        },
        actions = {
            TextButton(
                onClick = onContinue,
                enabled = buttonState == ButtonState.Enabled,
                colors = ButtonDefaults.textButtonColors().copy(contentColor = MaterialTheme.colorScheme.primary),
            ) { Text(stringResource(R.string.common_continue).uppercase()) }
        },
        content = content,
    )
}
