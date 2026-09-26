package com.gemwallet.android.features.transactions.presents.transaction

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.features.transactions.presents.transaction.components.TransactionSwapProgressItem
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionHeaderTarget
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.TransactionListHead
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemTransactionDetailRow

@Composable
internal fun TransactionScene(title: String, sections: List<ListSection<GemTransactionDetailRow>>, headerTarget: TransactionHeaderTarget?, chain: Chain, onAction: (TransactionAction) -> Unit) {
    val context = LocalContext.current
    Scene(
        title = title,
        actions = {
            IconButton(onClick = { onAction(TransactionAction.Share) }) {
                Icon(AppIcons.Share, "")
            }
        },
        onClose = { onAction(TransactionAction.Close) },
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            listSections(sections) { position, row ->
                when (row) {
                    is GemTransactionDetailRow.Participant -> AddressPropertyItem(
                        row = row.row,
                        listPosition = position,
                        onClick = { onAction(TransactionAction.OpenAddress(ChainAddress(row.row.chain.toChain(), row.row.address))) },
                    )

                    is GemTransactionDetailRow.Fee -> GemListRowView(
                        row = row.row,
                        listPosition = position,
                        modifier = Modifier.clickable { onAction(TransactionAction.ShowFeeDetails) },
                        accessory = { DataBadgeChevron() },
                    )

                    is GemTransactionDetailRow.SwapProgress -> TransactionSwapProgressItem(row.progress)

                    is GemTransactionDetailRow.Row -> GemListRowView(
                        row = row.row,
                        listPosition = position,
                        onSelectAddress = { address -> onAction(TransactionAction.OpenAddress(ChainAddress(chain, address))) },
                    )

                    is GemTransactionDetailRow.Header -> TransactionListHead(
                        header = row.header,
                        onClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                        onAssetClick = { onAction(TransactionAction.OpenAsset(it)) },
                    )

                    is GemTransactionDetailRow.SwapAgain -> MainActionButton(
                        title = row.title.string(context),
                        modifier = Modifier.padding(horizontal = padding16, vertical = paddingSmall),
                        onClick = {
                            onAction(
                                TransactionAction.OpenSwap(
                                    fromAssetId = AssetId(row.swap.fromAssetId),
                                    toAssetId = AssetId(row.swap.toAssetId),
                                ),
                            )
                        },
                    )
                }
            }
        }
    }
}

private fun TransactionHeaderTarget.navigation(): TransactionAction.Navigation = when (this) {
    is TransactionHeaderTarget.Asset -> TransactionAction.OpenAsset(assetId)
    is TransactionHeaderTarget.Nft -> TransactionAction.OpenNft(assetId)
    is TransactionHeaderTarget.Swap -> TransactionAction.OpenSwap(fromAssetId = fromAssetId, toAssetId = toAssetId)
    is TransactionHeaderTarget.Perpetual -> TransactionAction.OpenPerpetual(assetId)
}
