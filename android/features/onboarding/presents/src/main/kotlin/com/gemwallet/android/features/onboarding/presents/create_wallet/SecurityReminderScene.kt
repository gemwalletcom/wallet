package com.gemwallet.android.features.onboarding.presents.create_wallet

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.onboarding.viewmodels.create_wallet.models.securityReminderListItems
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.CenteredDescriptionText
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.introRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.GemSecurityReminderKind

@Composable
fun SecurityReminderScene(
    title: String = stringResource(R.string.wallet_new_title),
    kind: GemSecurityReminderKind = GemSecurityReminderKind.PHRASE,
    onAccept: () -> Unit,
    onCancel: CancelAction,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val items = remember(kind) { securityReminderListItems(context, kind) }

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
                { uriHandler.open(context, AppUrl.docs(kind.docsUrl())) },
            ) {
                Icon(AppIcons.InfoOutlined, "")
            }
        },
        onClose = { onCancel() },
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
                text = stringResource(kind.introRes()),
            )
            items.forEach { item ->
                SecurityReminderItem(item)
            }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }
}

@Composable
private fun SecurityReminderItem(item: ListItemModel) {
    ListItem(
        listPosition = ListPosition.Single,
        leading = item.image?.let { image -> { ListItemImageView(image = image) } },
        title = { Text(text = item.title, style = MaterialTheme.typography.bodyLarge) },
        subtitle = item.titleExtra?.let { description ->
            { Text(text = description, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.secondary) }
        },
    )
}

private fun GemSecurityReminderKind.docsUrl(): DocsUrl = when (this) {
    GemSecurityReminderKind.PHRASE -> DocsUrl.WhatIsSecretPhrase
    GemSecurityReminderKind.PRIVATE_KEY, GemSecurityReminderKind.ACCOUNT_KEY -> DocsUrl.WhatIsPrivateKey
}

@Preview
@Composable
fun PreviewSecurityReminderScene() {
    WalletTheme {
        SecurityReminderScene(onAccept = {}, onCancel = {})
    }
}
