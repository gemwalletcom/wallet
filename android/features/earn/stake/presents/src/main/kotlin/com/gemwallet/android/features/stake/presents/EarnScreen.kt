@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.stake.presents

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
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.features.stake.viewmodels.EarnViewModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.DelegationItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.subtitleSymbol
import com.gemwallet.android.ui.theme.paddingLarge

@Composable
fun EarnScreen(
    amountAction: AmountTransactionAction,
    onDelegation: (String, String) -> Unit,
    onCancel: () -> Unit,
    viewModel: EarnViewModel = hiltViewModel(),
) {
    val assetInfo by viewModel.assetInfo.collectAsStateWithLifecycle()
    val positions by viewModel.positions.collectAsStateWithLifecycle()
    val validatorRows by viewModel.validatorRows.collectAsStateWithLifecycle()
    val apr by viewModel.apr.collectAsStateWithLifecycle()
    val depositParams by viewModel.depositParams.collectAsStateWithLifecycle()
    val inSync by viewModel.isSync.collectAsStateWithLifecycle()

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
                item {
                    CenteredListHead(
                        title = earnAssetInfo.asset.name,
                        subtitle = earnAssetInfo.asset.subtitleSymbol,
                        leading = { HeaderIcon(earnAssetInfo.asset) },
                    )
                }

                item {
                    PropertyItem(
                        title = stringResource(R.string.stake_apr, ""),
                        data = apr.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
                        dataColor = MaterialTheme.colorScheme.tertiary,
                        listPosition = ListPosition.Single,
                    )
                }

                depositParams?.let { params ->
                    item {
                        PropertyItem(
                            action = R.string.wallet_deposit,
                            listPosition = ListPosition.Single,
                            onClick = { amountAction(params) },
                        )
                    }
                }

                if (positions.isEmpty()) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        EmptyContentView(type = EmptyContentType.Earn(symbol = earnAssetInfo.asset.symbol))
                    }
                } else {
                    item { SubheaderItem(R.string.perpetual_positions) }
                    itemsIndexed(positions) { index, item ->
                        DelegationItem(
                            assetInfo = earnAssetInfo,
                            delegation = item,
                            validator = validatorRows[item.validator.id] ?: return@itemsIndexed,
                            listPosition = ListPosition.getPosition(index, positions.size),
                            onClick = { onDelegation(item.validator.id, item.base.delegationId) },
                        )
                    }
                }
            }
        }
    }
}
