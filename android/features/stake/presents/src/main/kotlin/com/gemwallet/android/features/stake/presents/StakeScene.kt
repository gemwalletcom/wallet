@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.stake.presents

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.stake.presents.components.stakeActions
import com.gemwallet.android.features.stake.viewmodels.models.StakeActionUIModel
import com.gemwallet.android.features.stake.viewmodels.models.StakeSectionUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.DelegationItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.paddingLarge
import com.wallet.core.primitives.AssetData
import uniffi.gemstone.GemAssetText
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemServiceException

@Composable
internal fun StakeScene(
    inSync: Boolean,
    assetInfo: AssetData,
    header: GemAssetText?,
    actions: List<StakeActionUIModel>,
    stakeInfoUrl: String?,
    sections: List<StakeSectionUIModel>,
    infoRows: List<GemListRow>,
    loadError: GemServiceException?,
    amountAction: AmountTransactionAction,
    onAction: (StakeAction) -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scene(
        title = stringResource(id = R.string.transfer_stake_title),
        onClose = { onAction(StakeAction.Cancel) },
        actions = {
            stakeInfoUrl?.let { url ->
                IconButton(onClick = { uriHandler.open(context, url) }) {
                    Icon(imageVector = AppIcons.InfoOutlined, contentDescription = null)
                }
            }
        },
    ) {
        PullToRefreshBox(
            isRefreshing = inSync,
            onRefresh = { onAction(StakeAction.Refresh) },
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

                stakeInfoSection(infoRows)

                sections.forEach { section ->
                    item { SubheaderItem(section.title) }
                    when (section) {
                        is StakeSectionUIModel.Manage -> stakeActions(
                            actions = actions,
                            assetId = assetInfo.asset.id,
                            amountAction = amountAction,
                            onConfirm = { onAction(StakeAction.Confirm(it)) },
                        )

                        is StakeSectionUIModel.Resources -> itemsIndexed(section.rows) { index, row ->
                            GemListRowView(row = row, listPosition = ListPosition.getPosition(index, section.rows.size))
                        }

                        is StakeSectionUIModel.Delegations -> itemsIndexed(section.rows) { index, item ->
                            DelegationItem(
                                item = item,
                                listPosition = ListPosition.getPosition(index, section.rows.size),
                                onClick = { onAction(StakeAction.OpenDelegation(item.delegation)) },
                            )
                        }
                    }
                }

                if (sections.none { it is StakeSectionUIModel.Delegations }) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        when (loadError) {
                            null -> EmptyContentView(type = EmptyContentType(GemEmptyStateKind.STAKE, symbol = assetInfo.asset.symbol))
                            else -> GemListRowView(row = GemListRow.Error(loadError), listPosition = ListPosition.Single)
                        }
                    }
                }
            }
        }
    }
}

private fun LazyListScope.stakeInfoSection(rows: List<GemListRow>) {
    itemsPositioned(rows) { position, row -> GemListRowView(row = row, listPosition = position) }
}
