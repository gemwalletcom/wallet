package com.gemwallet.android.ui.components.screen

import androidx.annotation.DrawableRes
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult
import androidx.compose.material3.SnackbarVisuals
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import com.gemwallet.android.ui.theme.middlePadding
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallIconSize

@Immutable
class IconSnackbarVisuals(
    override val message: String,
    @param:DrawableRes val iconRes: Int,
) : SnackbarVisuals {
    override val actionLabel: String? = null
    override val withDismissAction: Boolean = false
    override val duration: SnackbarDuration = SnackbarDuration.Short
}

suspend fun SnackbarHostState.showSnackbar(
    message: String,
    @DrawableRes iconRes: Int,
): SnackbarResult = showSnackbar(IconSnackbarVisuals(message, iconRes))

@Composable
fun rememberSnackbarState(
    message: String?,
    @DrawableRes iconRes: Int,
    onShown: () -> Unit = {},
): SnackbarHostState {
    val snackbarHostState = remember { SnackbarHostState() }
    LaunchedEffect(message) {
        if (!message.isNullOrEmpty()) {
            snackbarHostState.showSnackbar(message, iconRes)
            onShown()
        }
    }
    return snackbarHostState
}

@Composable
fun SnackbarHost(hostState: SnackbarHostState) {
    androidx.compose.material3.SnackbarHost(hostState = hostState) { data ->
        val containerColor = MaterialTheme.colorScheme.scrim
        val contentColor = MaterialTheme.colorScheme.onSurface

        when (val visuals = data.visuals) {
            is IconSnackbarVisuals -> Snackbar(
                modifier = Modifier.middlePadding(),
                containerColor = containerColor,
                contentColor = contentColor,
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                ) {
                    Icon(
                        painter = painterResource(visuals.iconRes),
                        contentDescription = null,
                        modifier = Modifier.size(smallIconSize),
                    )
                    Text(text = visuals.message)
                }
            }

            else -> Snackbar(
                snackbarData = data,
                containerColor = containerColor,
                contentColor = contentColor,
                actionContentColor = MaterialTheme.colorScheme.primary,
            )
        }
    }
}
