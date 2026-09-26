package com.gemwallet.android.features.assets.presents.add

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.height
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.AppUrl
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.DocsInfoButton
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.fields.AddressChainField
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.Asset
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.GemListSection

private val networkItemHeight = 64.dp

@Composable
internal fun AddAssetScene(
    isSearching: Boolean,
    addressState: MutableState<String>,
    network: Asset?,
    sections: List<GemListSection>,
    verificationWarningRow: ListItemModel?,
    buttonState: ButtonState,
    canSelectChain: Boolean,
    snackbar: SnackbarHostState? = null,
    onAction: (AddAssetAction) -> Unit,
) {
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current

    Scene(
        title = stringResource(id = R.string.wallet_add_token_title),
        snackbar = snackbar,
        actions = {
            DocsInfoButton(AppUrl.docs(DocsUrl.AddCustomToken))
        },
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                state = buttonState,
                onClick = { onAction(AddAssetAction.Add) },
            )
        },
        onClose = { onAction(AddAssetAction.Cancel) },
    ) {
        SubheaderItem(R.string.transfer_network)
        if (network != null) {
            ChainItem(
                modifier = Modifier.height(networkItemHeight),
                title = network.chain.networkName(),
                icon = network.chain,
                onClick = if (canSelectChain) {
                    { onAction(AddAssetAction.SelectChain) }
                } else {
                    null
                },
                listPosition = ListPosition.Single,
                trailing = if (canSelectChain) {
                    { DataBadgeChevron() }
                } else {
                    null
                },
            )
        }
        Column {
            AddressChainField(
                label = stringResource(R.string.wallet_import_contract_address_field),
                value = addressState.value,
                onValueChange = { input ->
                    addressState.value = input
                },
                onQrScanner = { onAction(AddAssetAction.Scan) },
            )
        }
        if (isSearching) {
            Box {
                CircularProgressIndicator16(modifier = Modifier.align(Alignment.Center))
            }
        }
        sections.forEach { section ->
            section.rows.forEachIndexed { index, row ->
                GemListRowView(row = row, listPosition = ListPosition.getPosition(index, section.rows.size))
            }
        }
        if (verificationWarningRow != null) {
            ListItem(
                model = verificationWarningRow,
                listPosition = ListPosition.Single,
                modifier = Modifier.clickable { uriHandler.open(context, AppUrl.tokenVerification) },
            )
        }
    }
}
