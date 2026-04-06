package com.gemwallet.android.features.create_wallet.views

import androidx.annotation.StringRes
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import androidx.core.net.toUri
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.CenteredDescriptionText
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Emoji
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.sceneContentPadding
import uniffi.gemstone.Config
import uniffi.gemstone.PublicUrl

private val emojiFontSize = 24.sp

@Composable
fun PhraseAlertDialog(
    onAccept: () -> Unit,
    onCancel: CancelAction,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scene(
        title = stringResource(R.string.wallet_new_title),
        mainAction = {
            MainActionButton(
                stringResource(R.string.common_continue),
                onClick = onAccept,
            )
        },
        actions = {
            IconButton(
                {
                    uriHandler.open(
                        context,
                        Config().getPublicUrl(PublicUrl.TERMS_OF_SERVICE).toUri()
                            .buildUpon()
                            .appendQueryParameter("utm_source", "gemwallet_android")
                            .build()
                            .toString()
                    )
                }
            ) {
                Icon(Icons.Outlined.Info, "")
            }
        },
        onClose = { onCancel() }
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(sceneContentPadding()),
            verticalArrangement = Arrangement.spacedBy(paddingDefault),
        ) {
            CenteredDescriptionText(stringResource(R.string.onboarding_security_create_wallet_intro_title))
            InfoBlock(
                Emoji.lock,
                R.string.onboarding_security_create_wallet_keep_safe_title,
                R.string.onboarding_security_create_wallet_keep_safe_subtitle,
            )
            InfoBlock(
                Emoji.warning,
                R.string.onboarding_security_create_wallet_do_not_share_title,
                R.string.onboarding_security_create_wallet_do_not_share_subtitle,
            )
            InfoBlock(
                Emoji.gem,
                R.string.onboarding_security_create_wallet_no_recovery_title,
                R.string.onboarding_security_create_wallet_no_recovery_subtitle,
            )
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }

}

@Composable
private fun InfoBlock(
    emoji: String,
    @StringRes title: Int,
    @StringRes description: Int,
) {
    Card(
        modifier = Modifier,
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
    ) {
        Row(modifier = Modifier.defaultPadding()) {
            Text(text = emoji, fontSize = emojiFontSize)
            Spacer16()
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = stringResource(title),
                    style = MaterialTheme.typography.titleMedium.let {
                        it.copy(
                            lineHeightStyle = it.lineHeightStyle?.copy(
                                alignment = LineHeightStyle.Alignment.Top,
                            )
                        )
                    }
                )
                Text(
                    text = stringResource(description),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.secondary,
                )
            }
        }
    }
}

@Preview
@Composable
fun PreviewPhraseAlertDialog() {
    WalletTheme {
        PhraseAlertDialog( {} ) {}
    }
}
