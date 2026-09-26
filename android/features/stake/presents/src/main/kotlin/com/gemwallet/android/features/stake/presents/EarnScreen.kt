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
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.stake.viewmodels.EarnViewModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.DelegationItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.theme.paddingLarge
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow

@Composable
fun EarnScreen(amountAction: AmountTransactionAction, onDelegation: (String, String) -> Unit, onConfirm: ConfirmTransactionAction, onCancel: () -> Unit, viewModel: EarnViewModel = hiltViewModel()) {
    val assetInfo by viewModel.assetInfo.collectAsStateWithLifecycle()
    val positions by viewModel.positionRows.collectAsStateWithLifecycle()
    val aprRow by viewModel.aprRow.collectAsStateWithLifecycle()
    val header by viewModel.header.collectAsStateWithLifecycle()
    val depositParams by viewModel.depositParams.collectAsStateWithLifecycle()
    val inSync by viewModel.isSync.collectAsStateWithLifecycle()
    val loadError by viewModel.loadError.collectAsStateWithLifecycle()

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
                header?.let { header ->
                    item {
                        CenteredListHead(
                            title = header.asset.name,
                            subtitle = header.subtitleSymbol,
                            leading = { HeaderIcon(header.icon) },
                        )
                    }
                }

                item { GemListRowView(row = aprRow, listPosition = ListPosition.Single) }

                depositParams?.let { params ->
                    item {
                        ListItem(
                            model = viewModel.depositListItem,
                            listPosition = ListPosition.Single,
                            modifier = Modifier.clickable { amountAction(params) },
                            accessory = { DataBadgeChevron() },
                        )
                    }
                }

                if (positions.isEmpty()) {
                    if (!inSync) {
                        item {
                            Spacer(modifier = Modifier.height(paddingLarge))
                            when (val error = loadError) {
                                null -> EmptyContentView(type = EmptyContentType(GemEmptyStateKind.EARN, symbol = earnAssetInfo.asset.symbol))
                                else -> GemListRowView(row = GemListRow.Error(error), listPosition = ListPosition.Single)
                            }
                        }
                    }
                } else {
                    item { SubheaderItem(R.string.perpetual_positions) }
                    itemsIndexed(positions) { index, item ->
                        DelegationItem(
                            item = item,
                            listPosition = ListPosition.getPosition(index, positions.size),
                            onClick = { viewModel.onPosition(item.delegation, onDelegation, amountAction, onConfirm) },
                        )
                    }
                }
            }
        }
    }
}
