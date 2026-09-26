package com.gemwallet.android.features.onboarding.presents.create_wallet

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.sceneContentPadding
import uniffi.gemstone.DocsUrl

@Composable
fun PhraseAlertDialog(title: String = stringResource(R.string.wallet_new_title), onAccept: () -> Unit, onCancel: CancelAction) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val items = remember { securityReminderListItems(context) }

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
                { uriHandler.open(context, AppUrl.docs(DocsUrl.WhatIsSecretPhrase)) },
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
        ) {
            CenteredDescriptionText(
                modifier = Modifier.padding(horizontal = sceneContentPadding()),
                text = stringResource(R.string.onboarding_security_create_wallet_intro_title),
            )
            items.forEach { item ->
                ListItem(model = item, listPosition = ListPosition.Single)
            }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
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
