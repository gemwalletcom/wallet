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
import com.gemwallet.android.domains.asset.subtitleSymbol
import com.gemwallet.android.features.stake.presents.components.stakeActions
import com.gemwallet.android.features.stake.viewmodels.models.StakeActionUIModel
import com.gemwallet.android.features.stake.viewmodels.models.StakeSectionUIModel
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.DelegationItem
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.energyItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.paddingLarge

@Composable
internal fun StakeScene(
    inSync: Boolean,
    assetInfo: AssetInfo,
    actions: List<StakeActionUIModel>,
    rewardsText: String,
    stakeInfoUrl: String?,
    sections: List<StakeSectionUIModel>,
    infoRows: List<ListItemModel>,
    amountAction: AmountTransactionAction,
    onAction: (StakeSceneAction) -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scene(
        title = stringResource(id = R.string.transfer_stake_title),
        onClose = { onAction(StakeSceneAction.Cancel) },
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
            onRefresh = { onAction(StakeSceneAction.Refresh) },
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                item {
                    CenteredListHead(
                        title = assetInfo.asset.name,
                        subtitle = assetInfo.asset.subtitleSymbol,
                        leading = { HeaderIcon(assetInfo.asset) },
                    )
                }

                stakeInfoSection(infoRows)

                sections.forEach { section ->
                    item { SubheaderItem(section.title) }
                    when (section) {
                        is StakeSectionUIModel.Manage -> stakeActions(
                        actions = actions,
                        assetId = assetInfo.id(),
                            amountAction = amountAction,
                            onRewards = { onAction(StakeSceneAction.ClaimRewards) },
                        )
                        is StakeSectionUIModel.Resources -> energyItem(assetInfo.balance.metadata)
                        is StakeSectionUIModel.Delegations -> itemsIndexed(section.rows) { index, item ->
                            DelegationItem(
                                assetInfo = assetInfo,
                                delegation = item.delegation,
                                validator = item.validator,
                                listPosition = ListPosition.getPosition(index, section.rows.size),
                                onClick = { onAction(StakeSceneAction.OpenDelegation(item.delegation)) }
                            )
                        }
                    }
                }

                if (sections.none { it is StakeSectionUIModel.Delegations }) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        EmptyContentView(type = EmptyContentType.Stake(symbol = assetInfo.asset.symbol))
                    }
                }
            }
        }
    }
}

private fun LazyListScope.stakeInfoSection(rows: List<ListItemModel>) {
    itemsPositioned(rows) { position, row -> ListItem(model = row, listPosition = position) }
}
