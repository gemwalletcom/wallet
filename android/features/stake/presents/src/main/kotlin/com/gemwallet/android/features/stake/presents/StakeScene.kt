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
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.stake.presents.components.stakeActions
import com.gemwallet.android.ui.R
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
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.paddingLarge
import com.wallet.core.primitives.AssetData
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemStakeSection
import uniffi.gemstone.GemStakeViewState

@Composable
internal fun StakeScene(inSync: Boolean, assetInfo: AssetData, state: GemStakeViewState, onAction: (StakeAction) -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scene(
        title = stringResource(id = R.string.transfer_stake_title),
        onClose = { onAction(StakeAction.Cancel) },
        actions = {
            state.docsUrl?.let { url ->
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
                item {
                    CenteredListHead(
                        title = state.asset.asset.name,
                        subtitle = state.asset.subtitleSymbol,
                        leading = { HeaderIcon(state.asset.icon) },
                    )
                }

                stakeInfoSection(state.infoRows)

                state.sections.forEach { section ->
                    item { SubheaderItem(section.stringRes()) }
                    when (section) {
                        GemStakeSection.MANAGE -> stakeActions(
                            actions = state.actions,
                            onOpen = { kind, destination -> onAction(StakeAction.Open(kind, destination)) },
                        )

                        GemStakeSection.RESOURCES -> itemsIndexed(state.resourceRows) { index, row ->
                            GemListRowView(row = row, listPosition = ListPosition.getPosition(index, state.resourceRows.size))
                        }

                        GemStakeSection.DELEGATIONS -> itemsIndexed(state.delegations) { index, item ->
                            DelegationItem(
                                row = item.row,
                                listPosition = ListPosition.getPosition(index, state.delegations.size),
                                onClick = { onAction(StakeAction.OpenDelegation(item.delegation.toPrimitives())) },
                            )
                        }
                    }
                }

                if (GemStakeSection.DELEGATIONS !in state.sections) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        when (val phase = state.delegationsPhase) {
                            null -> GemListRowView(row = GemListRow.Loading, listPosition = ListPosition.Single)
                            is GemListPhase.Empty -> EmptyContentView(state = phase.state, symbol = assetInfo.asset.symbol)
                            is GemListPhase.Error -> GemListRowView(row = GemListRow.Error(phase.error), listPosition = ListPosition.Single)
                            GemListPhase.Rows -> Unit
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
