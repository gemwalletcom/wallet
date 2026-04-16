@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.stake.presents

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.lockTime
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.ui.models.subtitleSymbol
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetInfo
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
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.features.stake.models.StakeAction
import com.gemwallet.android.features.stake.presents.components.stakeActions
import com.wallet.core.primitives.AssetId
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
    amountAction: AmountTransactionAction,
    onRefresh: () -> Unit,
    onRewards: () -> Unit,
    onDelegation: (String, String) -> Unit,
    onCancel: () -> Unit,
) {
    val pullToRefreshState = rememberPullToRefreshState()

    Scene(
        title = stringResource(id = R.string.transfer_stake_title),
        onClose = onCancel,
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
                minAmount(assetInfo.asset.chain)
                apr(assetInfo.stakeApr ?: 0.0)
                lockTime(assetInfo.lockTime, assetInfo.id())

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
                    itemsIndexed(delegations) { index, item ->
                        DelegationItem(
                            assetInfo = assetInfo,
                            delegation = item,
                            completedAt = availableIn(item),
                            listPosition = ListPosition.getPosition(index, delegations.size),
                            onClick = { onDelegation(item.validator.id, item.base.delegationId) }
                        )
                    }
                }
            }
        }
    }
}

private fun LazyListScope.lockTime(lockTime: Int?, id: AssetId) {
    lockTime ?: return
    item {
        PropertyItem(
            title = stringResource(id = R.string.stake_lock_time),
            data = "$lockTime days",
            info = InfoSheetEntity.StakeLockTimeInfo(icon = id.getIconUrl()),
            listPosition = ListPosition.Last,
        )
    }
}

internal fun LazyListScope.apr(apr: Double) {
    item {
        PropertyItem(
            title = stringResource(id = R.string.stake_apr, ""),
            data = apr.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
            listPosition = ListPosition.Middle
        )
    }
}

internal fun LazyListScope.minAmount(chain: Chain) {
    val value = Config().getStakeConfig(chain.string).minAmount.toLong()
    if (value <= 0) {
        return
    }
    item {
        PropertyItem(
            title = stringResource(id = R.string.stake_minimum_amount, ""),
            data = chain.asset().format(Crypto(value.toBigInteger()), decimalPlace = 2),
            listPosition = ListPosition.First
        )
    }
}
