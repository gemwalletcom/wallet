package com.gemwallet.android

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyAction
import com.gemwallet.android.ui.components.empty.EmptyStateView

private fun Modifier.consumeAllPointerEvents(): Modifier = pointerInput(Unit) {
    awaitPointerEventScope {
        while (true) {
            awaitPointerEvent().changes.forEach { it.consume() }
        }
    }
}

@Composable
internal fun LockedSplash() {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surface)
            .consumeAllPointerEvents()
    ) {
        Image(
            modifier = Modifier
                .align(Alignment.Center),
            painter = painterResource(id = R.drawable.ic_splash_screen),
            contentDescription = null,
        )
    }
}

@Composable
internal fun SystemAuthEnrollmentRequired(
    onOpenSettings: () -> Unit,
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surface)
            .consumeAllPointerEvents(),
    ) {
        EmptyStateView(
            modifier = Modifier.align(Alignment.Center),
            title = stringResource(R.string.settings_security_authentication),
            buttons = listOf(
                EmptyAction(
                    title = stringResource(R.string.common_open_settings),
                    onClick = onOpenSettings,
                )
            ),
        )
    }
}
