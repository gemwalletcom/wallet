package com.gemwallet.android.features.create_wallet.views

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
import androidx.compose.ui.unit.sp
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.CenteredDescriptionText
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Emoji
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.sceneContentPadding
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.GemPrivateKeyScope
import uniffi.gemstone.GemSecurityReminder
import uniffi.gemstone.GemSecurityReminderItem

private val emojiFontSize = 24.sp

internal data class SecurityReminderItem(
    val title: String,
    val subtitle: String? = null,
    val emoji: String,
)

@Composable
internal fun GemSecurityReminder.items(privateKeyChain: String? = null): List<SecurityReminderItem> =
    items.map { it.toItem(privateKeyChain) }

@Composable
private fun GemSecurityReminderItem.toItem(privateKeyChain: String?): SecurityReminderItem = when (this) {
    is GemSecurityReminderItem.KeepSafe -> SecurityReminderItem(
        title = stringResource(R.string.onboarding_security_create_wallet_keep_safe_title),
        subtitle = if (privateKeyChain == null) stringResource(R.string.onboarding_security_create_wallet_keep_safe_subtitle) else null,
        emoji = Emoji.lock,
    )
    is GemSecurityReminderItem.DoNotShare -> SecurityReminderItem(
        title = stringResource(R.string.onboarding_security_create_wallet_do_not_share_title),
        subtitle = if (privateKeyChain == null) {
            stringResource(R.string.onboarding_security_create_wallet_do_not_share_subtitle)
        } else {
            stringResource(R.string.private_key_do_not_share_description, privateKeyChain)
        },
        emoji = Emoji.warning,
    )
    is GemSecurityReminderItem.NoRecovery -> SecurityReminderItem(
        title = stringResource(R.string.onboarding_security_create_wallet_no_recovery_title),
        subtitle = stringResource(R.string.onboarding_security_create_wallet_no_recovery_subtitle),
        emoji = Emoji.gem,
    )
    is GemSecurityReminderItem.KeyScope -> when (scope) {
        GemPrivateKeyScope.ONE_CHAIN -> SecurityReminderItem(
            title = stringResource(R.string.private_key_one_chain_title),
            subtitle = privateKeyChain?.let { stringResource(R.string.private_key_one_chain_description, it) },
            emoji = Emoji.gem,
        )
        GemPrivateKeyScope.EVM_CHAINS -> sharedChainsItem(stringResource(R.string.private_key_scope_evm))
        GemPrivateKeyScope.COSMOS_CHAINS -> sharedChainsItem(stringResource(R.string.private_key_scope_cosmos))
    }
}

@Composable
private fun sharedChainsItem(chains: String) = SecurityReminderItem(
    title = stringResource(R.string.private_key_shared_chains_title),
    subtitle = stringResource(R.string.private_key_shared_chains_description, chains),
    emoji = Emoji.gem,
)

@Composable
internal fun SecurityReminderScene(
    title: String,
    intro: String,
    items: List<SecurityReminderItem>,
    docsUrl: DocsUrl,
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
                { uriHandler.open(context, AppUrl.docs(docsUrl)) }
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
            CenteredDescriptionText(intro)
            items.forEach { item -> InfoBlock(item) }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }
}

@Composable
private fun InfoBlock(item: SecurityReminderItem) {
    Card(
        modifier = Modifier,
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
    ) {
        Row(modifier = Modifier.defaultPadding()) {
            Text(text = item.emoji, fontSize = emojiFontSize)
            Spacer16()
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = item.title,
                    style = MaterialTheme.typography.titleMedium.let {
                        it.copy(
                            lineHeightStyle = it.lineHeightStyle?.copy(
                                alignment = LineHeightStyle.Alignment.Top,
                            )
                        )
                    }
                )
                item.subtitle?.let {
                    Text(
                        text = it,
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.secondary,
                    )
                }
            }
        }
    }
}
