package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalUriHandler
import androidx.annotation.StringRes
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.nft.presents.components.NftHeaderActions
import com.gemwallet.android.features.nft.presents.components.NftTitle
import com.gemwallet.android.features.nft.viewmodels.NftDetailsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.components.list_item.property.toSocialLinks
import uniffi.gemstone.socialLinks
import com.gemwallet.android.ui.components.list_item.property.verificationStatusItem
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.ReportReason
import kotlinx.coroutines.launch
import uniffi.gemstone.GemCollectibleAttributeValue
import uniffi.gemstone.GemCollectibleIdentifier
import uniffi.gemstone.GemCollectibleRow
import uniffi.gemstone.GemCollectibleSection
import java.text.DateFormat
import java.util.Date

@Composable
fun NFTDetailsScene(
    cancelAction: CancelAction,
    onRecipient: (AssetId, NFTAssetId) -> Unit,
) {
    val viewModel: NftDetailsViewModel = hiltViewModel()
    val assetData by viewModel.nftAsset.collectAsStateWithLifecycle()

    val uriHandler = LocalUriHandler.current
    val snackbar = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val refresh = stringResource(R.string.common_refresh)
    val refreshFailed = stringResource(R.string.errors_error_occurred)

    val model = assetData ?: return
    var isReportVisible by remember { mutableStateOf(false) }
    val reported = stringResource(R.string.transaction_status_confirmed)
    val avatarSet = stringResource(R.string.nft_set_as_avatar)
    Scene(
        titleContent = {
            NftTitle(
                name = model.asset.name,
                status = model.collection.status,
                iconSize = compactIconSize,
            )
        },
        onClose = { cancelAction() },
        snackbar = snackbar,
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            item {
                NftImage(
                    source = model.asset.toImageSource(),
                    modifier = Modifier
                        .padding(horizontal = sceneContentPadding())
                        .fillMaxWidth()
                        .aspectRatio(1f)
                        .clip(RoundedCornerShape(paddingDefault)),
                )
            }
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = paddingDefault, bottom = paddingSmall),
                    contentAlignment = Alignment.Center,
                ) {
                    NftHeaderActions(
                        canSend = model.details.canSend,
                        onSend = { onRecipient(AssetId(model.asset.chain), model.asset.id) },
                        onRefresh = {
                            scope.launch {
                                if (viewModel.refresh()) {
                                    snackbar.showSnackbar(refresh, R.drawable.ic_check_circle)
                                } else {
                                    snackbar.showSnackbar(refreshFailed, R.drawable.ic_error)
                                }
                            }
                        },
                        onSetAsAvatar = {
                            scope.launch {
                                if (viewModel.setAsAvatar()) {
                                    snackbar.showSnackbar(avatarSet, R.drawable.ic_check_circle)
                                } else {
                                    snackbar.showSnackbar(refreshFailed, R.drawable.ic_error)
                                }
                            }
                        },
                        onReport = { isReportVisible = true },
                    )
                }
            }
            model.details.sections.forEach { section ->
                when (section) {
                    is GemCollectibleSection.Status -> verificationStatusItem(section.status.toPrimitives())
                    is GemCollectibleSection.Info -> itemsPositioned(section.rows) { position, row -> InfoRow(row, position) }
                    is GemCollectibleSection.Attributes -> nftAttributes(section.attributes.map { it.name to it.value.text() })
                    is GemCollectibleSection.Links -> nftLinks(section.links.map { it.toPrimitives() }) { uriHandler.openUri(it) }
                }
            }
        }
    }
    ReportReasonSheet(
        isVisible = isReportVisible,
        onDismiss = { isReportVisible = false },
        onSelect = { reason ->
            scope.launch {
                if (viewModel.report(reason)) {
                    snackbar.showSnackbar(reported, R.drawable.ic_check_circle)
                } else {
                    snackbar.showSnackbar(refreshFailed, R.drawable.ic_error)
                }
            }
        },
    )
}

@Composable
private fun ReportReasonSheet(
    isVisible: Boolean,
    onDismiss: () -> Unit,
    onSelect: (ReportReason) -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.nft_report_report_button_title),
    ) {
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            itemsPositioned(ReportReason.entries) { position, reason ->
                ListItem(
                    modifier = Modifier.clickable {
                        onSelect(reason)
                        onDismiss()
                    },
                    minHeight = ListItemDefaults.plainMinHeight,
                    title = { ListItemTitleText(stringResource(reason.titleRes)) },
                    listPosition = position,
                )
            }
        }
    }
}

private val ReportReason.titleRes: Int
    get() = when (this) {
        ReportReason.Spam -> R.string.nft_report_reason_spam
        ReportReason.Malicious -> R.string.nft_report_reason_malicious
        ReportReason.Inappropriate -> R.string.nft_report_reason_inappropriate
        ReportReason.Copyright -> R.string.nft_report_reason_copyright
        ReportReason.Other -> R.string.transfer_other_title
    }

@Composable
private fun InfoRow(row: GemCollectibleRow, position: ListPosition) {
    when (row) {
        is GemCollectibleRow.Collection -> PropertyItem(R.string.nft_collection, row.name, listPosition = position)
        is GemCollectibleRow.Network -> PropertyNetworkItem(row.chain.toChain(), listPosition = position)
        is GemCollectibleRow.Contract -> IdentifierRow(R.string.asset_contract, row.identifier, position)
        is GemCollectibleRow.TokenId -> IdentifierRow(R.string.asset_token_id, row.identifier, position)
    }
}

@Composable
private fun IdentifierRow(@StringRes title: Int, identifier: GemCollectibleIdentifier, position: ListPosition) {
    AddressPropertyItem(
        title = title,
        displayText = identifier.text,
        copyValue = identifier.value,
        explorerLink = identifier.explorer?.toPrimitives(),
        listPosition = position,
    )
}

private fun GemCollectibleAttributeValue.text(): String = when (this) {
    is GemCollectibleAttributeValue.Text -> value
    is GemCollectibleAttributeValue.Date -> DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(date))
}

private fun LazyListScope.nftAttributes(attributes: List<Pair<String, String>>) {
    item {
        SubheaderItem(R.string.nft_properties)
    }
    itemsPositioned(attributes) { position, (name, value) ->
        PropertyItem(name, value, listPosition = position)
    }
}

private fun LazyListScope.nftLinks(links: List<AssetLink>, onLinkClick: (String) -> Unit) {
    val models = socialLinks(links.map { it.toGem() }).toSocialLinks()
    if (models.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.social_links)
    }
    itemsPositioned(models) { position, item ->
        PropertyItem(
            action = item.label,
            actionIconModel = item.icon,
            data = item.host,
            listPosition = position,
        ) { onLinkClick(item.url) }
    }
}
