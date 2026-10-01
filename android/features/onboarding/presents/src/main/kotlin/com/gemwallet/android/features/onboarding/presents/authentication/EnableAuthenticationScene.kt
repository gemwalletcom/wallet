package com.gemwallet.android.features.onboarding.presents.authentication

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.empty.StateHeroView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import uniffi.gemstone.enableAuthenticationLabel

@Composable
fun EnableAuthenticationScene(onEnable: () -> Unit, onSkip: () -> Unit) {
    val context = LocalContext.current

    BackHandler(onBack = onSkip)

    Scene(
        title = "",
        mainAction = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                MainActionButton(
                    title = enableAuthenticationLabel(null).string(context),
                    onClick = { context.requestAuth(AuthRequest.Required, onSuccess = onEnable) },
                )
                TextButton(
                    onClick = onSkip,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = paddingDefault),
                ) {
                    Text(
                        text = stringResource(R.string.common_skip),
                        color = MaterialTheme.colorScheme.secondary,
                        style = MaterialTheme.typography.bodyLarge,
                    )
                }
            }
        },
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            StateHeroView(
                icon = AppIcons.Lock,
                title = stringResource(R.string.lock_passcode),
                description = stringResource(R.string.lock_footer),
            )
        }
    }
}

@Preview
@Composable
fun PreviewEnableAuthenticationScene() {
    WalletTheme {
        EnableAuthenticationScene(onEnable = {}, onSkip = {})
    }
}
