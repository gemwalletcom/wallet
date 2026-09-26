@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.stake.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.stake.viewmodels.EarnViewModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.DelegationItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.theme.paddingLarge
import uniffi.gemstone.GemEarnSection
import uniffi.gemstone.GemEmptyStateKind

@Composable
fun EarnScreen(amountAction: AmountTransactionAction, onDelegation: (String, String) -> Unit, onConfirm: ConfirmTransactionAction, onCancel: () -> Unit, viewModel: EarnViewModel = hiltViewModel()) {
    val assetInfo by viewModel.assetInfo.collectAsStateWithLifecycle()
    val earnView by viewModel.earnView.collectAsStateWithLifecycle()
    val depositParams by viewModel.depositParams.collectAsStateWithLifecycle()
    val inSync by viewModel.isSync.collectAsStateWithLifecycle()
    val context = LocalContext.current

    val earnAssetInfo = assetInfo
    if (earnAssetInfo == null) {
        LoadingScene(title = stringResource(R.string.common_earn), onCancel = onCancel)
        return
    }

    Scene(
        title = stringResource(R.string.common_earn),
        onClose = onCancel,
    ) {
        PullToRefreshBox(
            isRefreshing = inSync,
            onRefresh = viewModel::onRefresh,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                val earn = earnView ?: return@LazyColumn
                item {
                    CenteredListHead(
                        title = earn.asset.asset.name,
                        subtitle = earn.asset.subtitleSymbol,
                        leading = { HeaderIcon(earn.asset.icon) },
                    )
                }

                item { GemListRowView(row = earn.rateRow, listPosition = ListPosition.Single) }

                earn.sections.forEach { section ->
                    item { SubheaderItem(section.stringRes()) }
                    when (section) {
                        GemEarnSection.MANAGE -> depositParams?.let { params ->
                            item {
                                ListItem(
                                    model = earn.depositRow.listItemModel(context) ?: ListItemModel(title = ""),
                                    listPosition = ListPosition.Single,
                                    modifier = Modifier.clickable { amountAction(params) },
                                    accessory = { DataBadgeChevron() },
                                )
                            }
                        }

                        GemEarnSection.POSITIONS -> itemsIndexed(earn.positions) { index, item ->
                            DelegationItem(
                                row = item.row,
                                listPosition = ListPosition.getPosition(index, earn.positions.size),
                                onClick = { viewModel.onPosition(item.delegation.toPrimitives(), onDelegation, amountAction, onConfirm) },
                            )
                        }
                    }
                }

                if (earn.showsEmpty) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        EmptyContentView(kind = GemEmptyStateKind.EARN, symbol = earnAssetInfo.asset.symbol)
                    }
                }
            }
        }
    }
}
