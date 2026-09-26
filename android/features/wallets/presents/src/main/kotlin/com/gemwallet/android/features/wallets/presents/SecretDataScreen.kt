package com.gemwallet.android.features.wallets.presents

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.wallets.viewmodels.SecretDataViewModel
import com.gemwallet.android.features.wallets.viewmodels.models.WalletSecretContentUIModel
import com.gemwallet.android.ui.DetectScreenshot
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.DocsInfoButton
import com.gemwallet.android.ui.components.buttons.CopyButton
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.PhraseLayout
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.sceneContentPaddingValues
import com.gemwallet.android.ui.theme.space8

@Composable
fun SecretDataScreen(onCancel: () -> Unit, viewModel: SecretDataViewModel = hiltViewModel()) {
    DisableScreenShooting()
    DetectScreenshot(AppUrl.howToSecureSecretPhrase)

    val result by viewModel.secret.collectAsStateWithLifecycle()
    val title = stringResource(viewModel.secretKind.stringRes())

    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()

    val secret = result?.getOrNull()
    if (secret == null) {
        if (result == null) {
            LoadingScene(title = title, onCancel)
        } else {
            SecretDataErrorScene(title = title, onCancel = onCancel)
        }
        return
    }

    Scene(
        title = title,
        padding = sceneContentPaddingValues(),
        actions = {
            DocsInfoButton(AppUrl.howToSecureSecretPhrase)
        },
        onClose = onCancel,
    ) {
        val warningHorizontalPadding = adaptivePadding(default = paddingDefault, compact = paddingMiddle)

        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(paddingDefault),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(
                        color = MaterialTheme.colorScheme.error.copy(alpha = alpha10),
                        shape = MaterialTheme.shapes.small,
                    )
                    .padding(horizontal = warningHorizontalPadding, vertical = paddingDefault),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(space8),
            ) {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(id = R.string.secret_phrase_do_not_share_title),
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.titleMedium,
                    textAlign = TextAlign.Center,
                )
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(id = R.string.secret_phrase_do_not_share_description),
                    color = MaterialTheme.colorScheme.error,
                    textAlign = TextAlign.Center,
                )
            }

            when (secret) {
                is WalletSecretContentUIModel.PrivateKey -> Text(
                    text = secret.key,
                    style = MaterialTheme.typography.titleMedium,
                    textAlign = TextAlign.Center,
                )

                is WalletSecretContentUIModel.Words -> PhraseLayout(rows = secret.rows)
            }

            CopyButton(onClick = { clipboardManager.setCopy(context, secret.copy()) })
        }
    }
}

@Composable
private fun SecretDataErrorScene(title: String, onCancel: () -> Unit) {
    Scene(
        title = title,
        padding = sceneContentPaddingValues(),
        onClose = onCancel,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
        ) {
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = stringResource(R.string.errors_keystore_access),
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
            )
        }
    }
}
