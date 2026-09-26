package com.gemwallet.android

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.components.consumeAllPointerEvents
import com.gemwallet.android.ui.components.empty.EmptyAction
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.R as UiR

@Composable
internal fun SystemAuthEnrollmentRequired(onOpenSettings: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surface)
            .consumeAllPointerEvents(),
    ) {
        EmptyStateView(
            modifier = Modifier.align(Alignment.Center),
            title = stringResource(UiR.string.settings_security_authentication),
            buttons = listOf(
                EmptyAction(
                    title = stringResource(UiR.string.common_open_settings),
                    onClick = onOpenSettings,
                ),
            ),
        )
    }
}
