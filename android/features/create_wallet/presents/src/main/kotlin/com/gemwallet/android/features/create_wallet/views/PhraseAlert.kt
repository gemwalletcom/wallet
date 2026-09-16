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
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.CenteredDescriptionText
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.create_wallet.localization.subtitleRes
import com.gemwallet.android.features.create_wallet.localization.titleRes
import com.gemwallet.android.features.create_wallet.style.emoji
import uniffi.gemstone.securityReminderItems
import uniffi.gemstone.DocsUrl

private val emojiFontSize = 24.sp

@Composable
fun PhraseAlertDialog(
    title: String = stringResource(R.string.wallet_new_title),
    onAccept: () -> Unit,
    onCancel: CancelAction,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scene(
        title = title,
        mainAction = {
            MainActionButton(
                stringResource(R.string.common_continue),
                onClick = onAccept,
            )
        },
        actions = {
            IconButton(
                { uriHandler.open(context, AppUrl.docs(DocsUrl.WhatIsSecretPhrase)) }
            ) {
                Icon(AppIcons.InfoOutlined, "")
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
            securityReminderItems().forEach { item ->
                InfoBlock(item.emoji(), item.titleRes(), item.subtitleRes())
            }
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
        PhraseAlertDialog(onAccept = {}, onCancel = {})
    }
}
