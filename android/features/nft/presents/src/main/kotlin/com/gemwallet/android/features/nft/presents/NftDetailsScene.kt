package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
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
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.nft.presents.components.NftHeaderActions
import com.gemwallet.android.features.nft.presents.components.NftTitle
import com.gemwallet.android.features.nft.viewmodels.NftDetailsViewModel
import com.gemwallet.android.features.nft.viewmodels.models.NftSectionUIModel
import com.gemwallet.android.features.nft.viewmodels.models.ReportReasonUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
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
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.ReportReason
import kotlinx.coroutines.launch

@Composable
fun NFTDetailsScene(cancelAction: CancelAction, onRecipient: (NFTAsset) -> Unit) {
    val viewModel: NftDetailsViewModel = hiltViewModel()
    val assetData by viewModel.nftAsset.collectAsStateWithLifecycle()

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
                isVerified = model.isVerified,
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
                        actions = model.actions,
                        onSend = { onRecipient(model.asset) },
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
            model.sections.forEach { section ->
                when (section) {
                    is NftSectionUIModel.Status -> verificationStatusItem(section.status)

                    is NftSectionUIModel.Info -> itemsPositioned(section.rows) { position, row -> GemListRowView(row = row, listPosition = position) }

                    is NftSectionUIModel.Attributes -> {
                        item { SubheaderItem(section.title) }
                        itemsPositioned(section.rows) { position, row -> ListItem(model = row, listPosition = position) }
                    }

                    is NftSectionUIModel.Links -> {
                        item { SubheaderItem(section.title) }
                        item { GemListRowView(row = section.row, listPosition = ListPosition.Single) }
                    }
                }
            }
        }
    }
    ReportReasonSheet(
        isVisible = isReportVisible,
        reasons = viewModel.reportReasons,
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
private fun ReportReasonSheet(isVisible: Boolean, reasons: List<ReportReasonUIModel>, onDismiss: () -> Unit, onSelect: (ReportReason) -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.nft_report_report_button_title),
    ) {
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            itemsPositioned(reasons) { position, item ->
                ListItem(
                    model = item.model,
                    listPosition = position,
                    modifier = Modifier.clickable {
                        onSelect(item.reason)
                        onDismiss()
                    },
                    minHeight = ListItemDefaults.plainMinHeight,
                )
            }
        }
    }
}
