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
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.features.nft.presents.components.NftHeaderActions
import com.gemwallet.android.features.nft.presents.components.NftTitle
import com.gemwallet.android.features.nft.viewmodels.NftDetailsViewModel
import com.gemwallet.android.ui.LocalAddressService
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
import com.gemwallet.android.ui.components.list_item.property.toSocialLinks
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
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.ReportReason
import kotlinx.coroutines.launch

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
    Scene(
        titleContent = {
            NftTitle(
                name = model.assetName,
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
                        canSend = model.canSend,
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
                        onReport = { isReportVisible = true },
                    )
                }
            }
            verificationStatusItem(model.collection.status)
            generalInfo(model)
            nftAttributes(model.attributes)
            nftLinks(model.collection.links) { uriHandler.openUri(it) }
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

private fun LazyListScope.generalInfo(model: NftAssetDetailsData) {
    item {
        PropertyItem(R.string.nft_collection, model.collection.name, listPosition = ListPosition.First)
        PropertyNetworkItem(model.collection.chain, listPosition = ListPosition.Middle)
        model.asset.contractAddress?.let {
            AddressPropertyItem(
                title = R.string.asset_contract,
                displayText = AddressFormatter(LocalAddressService.current, it, chain = model.collection.chain).value(),
                copyValue = it,
                explorerLink = model.contractExplorerLink,
                listPosition = ListPosition.Middle,
            )
        }
        val tokenId = model.asset.tokenId
        val tokenIdDisplayText = if (tokenId.length > 16) {
            AddressFormatter(LocalAddressService.current, tokenId, chain = model.collection.chain).value()
        } else {
            "#$tokenId"
        }
        AddressPropertyItem(
            title = R.string.asset_token_id,
            displayText = tokenIdDisplayText,
            copyValue = tokenId,
            explorerLink = model.tokenIdExplorerLink,
            listPosition = ListPosition.Last,
        )
    }
}

private fun LazyListScope.nftAttributes(attributes: List<NFTAttribute>) {
    item {
        SubheaderItem(R.string.nft_properties)
    }
    itemsPositioned(attributes.map(::NftAttributeUIModel)) { position, item ->
        PropertyItem(item.name, item.value, listPosition = position)
    }
}

private fun LazyListScope.nftLinks(links: List<AssetLink>, onLinkClick: (String) -> Unit) {
    val models = links.toSocialLinks()
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
