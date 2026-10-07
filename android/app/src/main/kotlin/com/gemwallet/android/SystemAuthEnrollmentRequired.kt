package com.gemwallet.android

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.consumeAllPointerEvents
import com.gemwallet.android.ui.components.empty.StateHeroView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.R as UiR

@Composable
internal fun SystemAuthEnrollmentRequired(onOpenSettings: () -> Unit) {
    Box(modifier = Modifier.fillMaxSize().consumeAllPointerEvents()) {
        Scene(
            title = "",
            mainAction = {
                MainActionButton(
                    title = stringResource(UiR.string.common_open_settings),
                    onClick = onOpenSettings,
                )
            },
        ) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                StateHeroView(
                    icon = AppIcons.Lock,
                    title = stringResource(UiR.string.lock_passcode_off_title),
                    description = stringResource(UiR.string.lock_passcode_off_description),
                )
            }
        }
    }
}
