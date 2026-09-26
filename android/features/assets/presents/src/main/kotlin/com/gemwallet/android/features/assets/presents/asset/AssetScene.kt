package com.gemwallet.android.features.assets.presents.asset

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.assets.presents.asset.components.AssetDetailRowItem
import com.gemwallet.android.features.assets.presents.asset.components.AssetDetailsMenu
import com.gemwallet.android.features.assets.presents.asset.components.AssetHeadItem
import com.gemwallet.android.features.assets.presents.asset.components.BannerItem
import com.gemwallet.android.features.assets.presents.asset.components.EmptyTransactionsItem
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.property.verificationStatusItem
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemTransactionRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun AssetScene(
    uiState: AssetUIState,
    transactions: List<GemTransactionRow>,
    transactionsErrorRow: GemListRow?,
    isRefreshing: Boolean,
    snackBar: SnackbarHostState = remember { SnackbarHostState() },
    onAction: (AssetAction) -> Unit,
) {
    val detailsState = uiState.details.state
    val swapAction: () -> Unit = {
        uiState.details.swapPair.payAssetId.toAssetId()?.let { payAssetId ->
            onAction(AssetAction.Swap(fromAssetId = payAssetId, toAssetId = uiState.details.swapPair.receiveAssetId?.toAssetId()))
        }
    }

    Scene(
        titleContent = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = uiState.details.title,
                    maxLines = 1,
                    overflow = TextOverflow.MiddleEllipsis,
                )
            }
        },
        progress = null,
        actions = {
            AssetDetailsMenu(
                uiState = uiState,
                priceAlert = uiState.priceAlertMenu,
                onPriceAlert = { onAction(AssetAction.TogglePriceAlert(it)) },
            )
        },
        onClose = { onAction(AssetAction.Close) },
        snackbar = snackBar,
    ) {
        val transactionSections = rememberDateSections(transactions) { it.createdAt }
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = { onAction(AssetAction.Refresh) },
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
            ) {
                item {
                    AssetHeadItem(header = uiState.details.header, onAction = onAction)
                }
                val banner = uiState.details.banner
                if (detailsState.showsBanners && banner != null) {
                    item {
                        BannerItem(
                            banner = banner,
                            onStake = { onAction(AssetAction.Stake(uiState.asset.id)) },
                            onActivate = { onAction(AssetAction.Confirm(ConfirmTransferInput(it))) },
                            onOpenPerpetuals = { onAction(AssetAction.OpenPerpetuals) },
                            onClose = { onAction(AssetAction.CloseBanner(it)) },
                        )
                    }
                }
                uiState.details.verificationStatus?.let { verificationStatusItem(it.toPrimitives()) }
                uiState.sections.forEach { section ->
                    section.title?.let { title -> item { SubheaderItem(title) } }
                    itemsPositioned(section.rows) { position, row ->
                        AssetDetailRowItem(row = row, listPosition = position, onAction = onAction)
                    }
                }
                item {
                    transactionsErrorRow?.let { GemListRowView(row = it, listPosition = ListPosition.Single) } ?: EmptyTransactionsItem(
                        size = transactions.size,
                        symbol = uiState.asset.symbol,
                        state = detailsState.emptyState,
                        onAction = { action ->
                            when (action) {
                                GemEmptyStateAction.BUY -> onAction(AssetAction.Buy(uiState.asset.id))
                                GemEmptyStateAction.SWAP -> swapAction()
                                GemEmptyStateAction.RECEIVE, GemEmptyStateAction.ADD_CUSTOM_TOKEN, GemEmptyStateAction.MANAGE_TOKEN_LIST, GemEmptyStateAction.CLEAR_FILTERS -> Unit
                            }
                        },
                    )
                }
                transactionsList(transactionSections) { onAction(AssetAction.OpenTransaction(it)) }
            }
        }
    }
}
