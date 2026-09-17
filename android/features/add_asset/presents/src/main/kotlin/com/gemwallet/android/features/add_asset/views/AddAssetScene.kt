package com.gemwallet.android.features.add_asset.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gemwallet.android.AppUrl
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.DocsInfoButton
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.fields.AddressChainField
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.LinkRowUIModel
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Emoji
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.space24
import com.wallet.core.primitives.Asset
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.GemAddAssetPhase

private val networkItemHeight = 64.dp

@Composable
internal fun AddAssetScene(
    searchState: GemAddAssetPhase,
    addressState: MutableState<String>,
    network: Asset?,
    token: Asset?,
    assetRows: List<ListItemModel>,
    explorerLink: LinkRowUIModel?,
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
                title = network.name,
                icon = network.chain,
                onClick = if (canSelectChain) {
                    { onAction(AddAssetAction.SelectChain) }
                } else null,
                listPosition = ListPosition.Single,
                trailing = if (canSelectChain) {
                    { DataBadgeChevron() }
                } else null
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
        if (searchState is GemAddAssetPhase.Loading) {
            Box {
                CircularProgressIndicator16(modifier = Modifier.align(Alignment.Center))
            }
        }
        if (searchState is GemAddAssetPhase.Failed) {
            Card(
                modifier = Modifier.padding(horizontal = sceneContentPadding()),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
            ) {
                Row(modifier = Modifier.defaultPadding()) {
                    Text(text = Emoji.warning, fontSize = space24.value.sp)
                    Spacer16()
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = stringResource(R.string.errors_error_occurred),
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Text(
                            text = stringResource(R.string.errors_token_invalid_id),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                }
            }
        }
        AssetInfoTable(token, assetRows)
        if (explorerLink != null && token != null) {
            ListItem(
                model = explorerLink.model,
                listPosition = ListPosition.Single,
                modifier = Modifier.clickable { uriHandler.open(context, explorerLink.url) },
                accessory = { DataBadgeChevron() },
            )
        }
        if (token != null) {
            Card(
                modifier = Modifier.padding(horizontal = sceneContentPadding()),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
                onClick = { uriHandler.open(context, AppUrl.tokenVerification) },
            ) {
                Row(modifier = Modifier.defaultPadding()) {
                    Text(text = Emoji.warning, fontSize = space24.value.sp)
                    Spacer16()
                    Column(modifier = Modifier.weight(1f)) {
                        Row {
                            Text(
                                text = stringResource(R.string.asset_verification_warning_title),
                                style = MaterialTheme.typography.titleMedium.let {
                                    it.copy(
                                        lineHeightStyle = it.lineHeightStyle?.copy(
                                            alignment = LineHeightStyle.Alignment.Top,
                                        )
                                    )
                                },
                            )
                            Spacer4()
                            Icon(
                                AppIcons.InfoOutlined, "",
                                modifier = Modifier.size(compactIconSize),
                                tint = MaterialTheme.colorScheme.secondary,
                            )
                        }
                        Text(
                            text = stringResource(R.string.asset_verification_warning_message),
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ColumnScope.AssetInfoTable(asset: Asset?, rows: List<ListItemModel>) {
    if (asset == null) {
        return
    }
    rows.forEachIndexed { index, row ->
        ListItem(model = row, listPosition = ListPosition.getPosition(index, rows.size))
    }
}
