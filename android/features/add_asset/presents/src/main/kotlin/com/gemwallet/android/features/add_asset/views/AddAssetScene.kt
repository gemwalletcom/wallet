package com.gemwallet.android.features.add_asset.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
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
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Emoji
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.space24
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.features.add_asset.viewmodels.models.TokenSearchState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import uniffi.gemstone.DocsUrl

@Composable
fun AddAssetScene(
    searchState: TokenSearchState,
    addressState: MutableState<String>,
    network: Asset,
    token: Asset?,
    explorerLink: BlockExplorerLink?,
    onScan: () -> Unit,
    onAddAsset: () -> Unit,
    onChainSelect: (() -> Unit)?,
    onCancel: () -> Unit,
) {
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current

    Scene(
        title = stringResource(id = R.string.wallet_add_token_title),
        actions = {
            IconButton(onClick = { uriHandler.open(context, AppUrl.docs(DocsUrl.AddCustomToken)) }) {
                Icon(Icons.Outlined.Info, "")
            }
        },
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                enabled = searchState is TokenSearchState.Idle && token != null,
                onClick = onAddAsset,
            )
        },
        onClose = onCancel,
    ) {
        SubheaderItem(R.string.transfer_network)
        ChainItem(
            modifier = Modifier.height(64.dp),
            title = network.name,
            icon = network.chain,
            onClick = onChainSelect,
            listPosition = ListPosition.Single,
            trailing = if (onChainSelect != null) {
                { DataBadgeChevron() }
            } else null
        )
        Column {
            AddressChainField(
                chain = network.chain,
                label = stringResource(R.string.wallet_import_contract_address_field),
                value = addressState.value,
                searchName = false,
                onValueChange = { input, _ ->
                    addressState.value = input
                },
                onQrScanner = onScan,
            )
        }
        if (searchState is TokenSearchState.Loading) {
            Box {
                CircularProgressIndicator16(modifier = Modifier.align(Alignment.Center))
            }
        }
        if (searchState is TokenSearchState.Error) {
            Card(
                modifier = Modifier.padding(horizontal = sceneContentPadding()),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
            ) {
                Row(modifier = Modifier.defaultPadding()) {
                    Text(text = Emoji.warning, fontSize = space24.value.sp)
                    Spacer16()
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = stringResource(R.string.errors_error_occured),
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
        AssetInfoTable(token)
        if (explorerLink != null && token != null) {
            PropertyItem(
                modifier = Modifier.clickable { uriHandler.open(context, explorerLink.link) },
                title = { PropertyTitleText(stringResource(R.string.transaction_view_on, explorerLink.name)) },
                data = { DataBadgeChevron() },
                listPosition = ListPosition.Single,
            )
        }
        if (token != null) {
            Card(
                modifier = Modifier.padding(horizontal = sceneContentPadding()),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
                onClick = { uriHandler.open(context, AppUrl.docs(DocsUrl.TokenVerification)) },
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
                                Icons.Outlined.Info, "",
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
private fun ColumnScope.AssetInfoTable(asset: Asset?) {
    if (asset == null) {
        return
    }
    PropertyItem(
        title = { PropertyTitleText(R.string.asset_name) },
        data = { PropertyDataText(asset.name, badge = { DataBadgeChevron(asset, false) }) },
        listPosition = ListPosition.First,
    )
    PropertyItem(
        R.string.asset_symbol,
        asset.symbol,
        listPosition = ListPosition.Middle,
    )
    PropertyItem(
        R.string.asset_decimals,
        asset.decimals.toString(),
        listPosition = ListPosition.Middle,
    )
    PropertyItem(
        R.string.common_type,
        asset.type.string,
        listPosition = ListPosition.Last,
    )
}

