@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.stake.presents

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.lockTime
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.models.subtitleSymbol
import com.gemwallet.android.ui.open
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.DelegationItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.availableIn
import com.gemwallet.android.ui.components.list_item.energyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.features.stake.models.StakeAction
import com.gemwallet.android.features.stake.presents.components.stakeActions
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Delegation
import uniffi.gemstone.Config

@Composable
fun StakeScene(
    inSync: Boolean,
    assetInfo: AssetInfo,
    actions: List<StakeAction>,
    isStakeEnabled: Boolean,
    delegations: List<Delegation>,
    stakeInfoUrl: String?,
    amountAction: AmountTransactionAction,
    onRefresh: () -> Unit,
    onRewards: () -> Unit,
    onDelegation: (Delegation) -> Unit,
    onCancel: () -> Unit,
) {
    val pullToRefreshState = rememberPullToRefreshState()
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scene(
        title = stringResource(id = R.string.transfer_stake_title),
        onClose = onCancel,
        actions = {
            stakeInfoUrl?.let { url ->
                IconButton(onClick = { uriHandler.open(context, url) }) {
                    Icon(imageVector = Icons.Outlined.Info, contentDescription = null)
                }
            }
        },
    ) {
        PullToRefreshBox(
            modifier = Modifier,
            isRefreshing = inSync,
            onRefresh = onRefresh,
            state = pullToRefreshState,
            indicator = {
                Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = inSync,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background
                )
            }
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                item {
                    CenteredListHead(
                        title = assetInfo.asset.name,
                        subtitle = assetInfo.asset.subtitleSymbol,
                        leading = { HeaderIcon(assetInfo.asset) },
                    )
                }

                stakeInfoSection(assetInfo)

                stakeActions(
                    actions = actions,
                    isStakeEnabled = isStakeEnabled,
                    assetId = assetInfo.id(),
                    amountAction = amountAction,
                    onRewards = onRewards,
                )

                energyItem(assetInfo.balance.metadata)

                if (delegations.isEmpty()) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        EmptyContentView(type = EmptyContentType.Stake(symbol = assetInfo.asset.symbol))
                    }
                } else {
                    item { SubheaderItem(R.string.stake_delegations) }
                    itemsIndexed(delegations) { index, item ->
                        DelegationItem(
                            assetInfo = assetInfo,
                            delegation = item,
                            completedAt = availableIn(item),
                            listPosition = ListPosition.getPosition(index, delegations.size),
                            onClick = { onDelegation(item) }
                        )
                    }
                }
            }
        }
    }
}

private sealed interface StakeInfoRow {
    data class MinAmount(val value: Long, val chain: Chain) : StakeInfoRow
    data class Apr(val value: Double, val iconUrl: String) : StakeInfoRow
    data class LockTime(val days: Int, val iconUrl: String) : StakeInfoRow
}

private fun LazyListScope.stakeInfoSection(assetInfo: AssetInfo) {
    val minAmountValue = Config().getStakeConfig(assetInfo.asset.chain.string).minAmount.toLong()
    val iconUrl = assetInfo.id().getIconUrl()
    val rows = listOfNotNull(
        minAmountValue.takeIf { it > 0 }?.let { StakeInfoRow.MinAmount(it, assetInfo.asset.chain) },
        StakeInfoRow.Apr(assetInfo.stakeApr ?: 0.0, iconUrl),
        assetInfo.lockTime?.let { StakeInfoRow.LockTime(it, iconUrl) },
    )
    itemsPositioned(rows) { position, row ->
        when (row) {
            is StakeInfoRow.MinAmount -> PropertyItem(
                title = stringResource(id = R.string.stake_minimum_amount, ""),
                data = row.chain.asset().format(Crypto(row.value.toBigInteger()), decimalPlace = 2),
                listPosition = position,
            )
            is StakeInfoRow.Apr -> PropertyItem(
                title = stringResource(id = R.string.stake_apr, ""),
                data = row.value.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
                dataColor = MaterialTheme.colorScheme.tertiary,
                info = InfoSheetEntity.StakeAprInfo(icon = row.iconUrl),
                listPosition = position,
            )
            is StakeInfoRow.LockTime -> PropertyItem(
                title = stringResource(id = R.string.stake_lock_time),
                data = "${row.days} days",
                info = InfoSheetEntity.StakeLockTimeInfo(icon = row.iconUrl),
                listPosition = position,
            )
        }
    }
}
