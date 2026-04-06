package com.gemwallet.android.features.wallet.presents

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
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet.viewmodels.WalletSecretDataViewModel
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.CopyButton
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.PhraseLayout
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.sceneContentPaddingValues
import com.gemwallet.android.ui.theme.space8
import com.wallet.core.primitives.WalletType

internal data class WalletSecretDataContent(
    val titleRes: Int,
    val warningTitleRes: Int,
    val warningDescriptionRes: Int,
)

internal fun walletSecretDataContent(walletType: WalletType): WalletSecretDataContent {
    return when (walletType) {
        WalletType.PrivateKey -> WalletSecretDataContent(
            titleRes = R.string.common_private_key,
            warningTitleRes = R.string.secret_phrase_do_not_share_title,
            warningDescriptionRes = R.string.secret_phrase_do_not_share_description,
        )
        WalletType.Multicoin,
        WalletType.Single,
        WalletType.View -> WalletSecretDataContent(
            titleRes = R.string.common_secret_phrase,
            warningTitleRes = R.string.secret_phrase_do_not_share_title,
            warningDescriptionRes = R.string.secret_phrase_do_not_share_description,
        )
    }
}

@Composable
fun WalletSecretDataNavScreen(
    onCancel: () -> Unit,
    viewModel: WalletSecretDataViewModel = hiltViewModel()
) {
    DisableScreenShooting()

    val value by viewModel.data.collectAsStateWithLifecycle()
    val walletType by viewModel.walletType.collectAsStateWithLifecycle()
    val content = walletSecretDataContent(walletType)

    val context = LocalContext.current
    val clipboardManager = LocalClipboard.current.nativeClipboard


    if (value == null) {
        LoadingScene(title = stringResource(id = content.titleRes), onCancel)
        return
    }

    Scene(
        title = stringResource(id = content.titleRes),
        padding = sceneContentPaddingValues(),
        backHandle = true,
        onClose = onCancel,
    ) {
        val warningHorizontalPadding = adaptivePadding(default = paddingDefault, compact = paddingMiddle)

        Column(
            modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(paddingDefault)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(
                        color = MaterialTheme.colorScheme.error.copy(alpha = alpha10),
                        shape = MaterialTheme.shapes.small
                    )
                    .padding(horizontal = warningHorizontalPadding, vertical = paddingDefault),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(space8)
            ) {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(id = content.warningTitleRes),
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.titleMedium,
                    textAlign = TextAlign.Center,
                )
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = stringResource(id = content.warningDescriptionRes),
                    color = MaterialTheme.colorScheme.error,
                    textAlign = TextAlign.Center,
                )
            }

            value?.privateKey()?.let {
                Text(
                    text = it,
                    style = MaterialTheme.typography.titleMedium,
                    textAlign = TextAlign.Center,
                )
            } ?: PhraseLayout(words = value?.phrase() ?: emptyList())

            CopyButton(onClick = { clipboardManager.setPlainText(context, value.toString(), true) })
        }
    }
}
