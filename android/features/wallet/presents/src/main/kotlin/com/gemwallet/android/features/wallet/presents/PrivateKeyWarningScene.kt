package com.gemwallet.android.features.wallet.presents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.CenteredDescriptionText
import com.gemwallet.android.ui.components.DocsInfoButton
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.subtitleRes
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.emoji
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.GemPrivateKeyWarningItem

@Composable
fun PrivateKeyWarningScene(onContinue: () -> Unit, onCancel: () -> Unit) {
    Scene(
        title = stringResource(R.string.common_private_key),
        onClose = onCancel,
        actions = {
            DocsInfoButton(AppUrl.docs(DocsUrl.WhatIsPrivateKey))
        },
        mainAction = {
            MainActionButton(
                title = stringResource(R.string.common_continue),
                onClick = onContinue,
            )
        },
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(top = sceneContentPadding()),
            verticalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            CenteredDescriptionText(
                modifier = Modifier.padding(start = sceneContentPadding(), end = sceneContentPadding(), bottom = paddingDefault),
                text = stringResource(R.string.private_key_warning_intro),
            )
            GemConstants.privateKeyWarningItems.forEach { item ->
                PrivateKeyWarningItem(item)
            }
        }
    }
}

@Composable
private fun PrivateKeyWarningItem(item: GemPrivateKeyWarningItem) {
    ListItem(
        listPosition = ListPosition.Single,
        leading = { ListItemImageView(image = ListItemImage.Emoji(item.emoji())) },
        title = { Text(text = stringResource(item.titleRes()), style = MaterialTheme.typography.bodyLarge) },
        subtitle = {
            Text(text = stringResource(item.subtitleRes()), style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.secondary)
        },
    )
}

@Preview
@Composable
private fun PrivateKeyWarningScenePreview() {
    WalletTheme {
        PrivateKeyWarningScene(onContinue = {}, onCancel = {})
    }
}
