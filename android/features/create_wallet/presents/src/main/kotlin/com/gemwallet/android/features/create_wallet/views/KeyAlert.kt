package com.gemwallet.android.features.create_wallet.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.Chain
import uniffi.gemstone.privateKeySecurityReminder

@Composable
fun KeyAlertDialog(
    chain: Chain,
    onAccept: () -> Unit,
    onCancel: CancelAction,
) {
    val reminder = remember(chain) { privateKeySecurityReminder(chain.string) }
    val chainName = chain.networkName()
    SecurityReminderScene(
        title = stringResource(R.string.common_private_key),
        intro = stringResource(R.string.private_key_reveal_intro, chainName),
        items = reminder.items(privateKeyChain = chainName),
        docsUrl = reminder.docsUrl,
        onAccept = onAccept,
        onCancel = onCancel,
    )
}

@Preview
@Composable
fun PreviewKeyAlertDialog() {
    WalletTheme {
        KeyAlertDialog(chain = Chain.Ethereum, onAccept = {}, onCancel = {})
    }
}
