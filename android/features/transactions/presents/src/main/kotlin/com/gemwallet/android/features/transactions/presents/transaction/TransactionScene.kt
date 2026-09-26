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
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.features.transactions.presents.transaction.components.TransactionSwapProgressItem
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionHeaderTarget
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionItemUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress

@Composable
internal fun TransactionScene(title: String, sections: List<ListSection<TransactionItemUIModel>>, headerTarget: TransactionHeaderTarget?, chain: Chain, onAction: (TransactionAction) -> Unit) {
    val uriHandler = LocalUriHandler.current
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
                    is TransactionItemUIModel.Item -> ListItem(
                        model = row.model,
                        listPosition = position,
                        modifier = row.url?.let { url -> Modifier.clickable { uriHandler.open(context, url) } } ?: Modifier,
                        accessory = row.url?.let { { DataBadgeChevron() } },
                    )

                    is TransactionItemUIModel.Address -> AddressPropertyItem(
                        title = row.title,
                        displayText = row.text,
                        copyValue = row.address,
                        explorerLink = row.explorerLink,
                        listPosition = position,
                        onClick = row.chain?.let { chain -> { onAction(TransactionAction.OpenAddress(ChainAddress(chain, row.address))) } },
                    )

                    is TransactionItemUIModel.Fee -> ListItem(
                        model = row.model,
                        listPosition = position,
                        modifier = Modifier.clickable { onAction(TransactionAction.ShowFeeDetails) },
                        accessory = { DataBadgeChevron() },
                    )

                    is TransactionItemUIModel.SwapProgress -> TransactionSwapProgressItem(row.model)

                    is TransactionItemUIModel.Row -> GemListRowView(
                        row = row.row,
                        listPosition = position,
                        onSelectAddress = { address -> onAction(TransactionAction.OpenAddress(ChainAddress(chain, address))) },
                    )

                    is TransactionItemUIModel.NftHead -> NftHead(
                        metadata = row.metadata,
                        onClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                    )

                    is TransactionItemUIModel.AmountHead -> AmountListHead(
                        icon = row.icon,
                        amount = row.amount,
                        equivalent = row.equivalent,
                        onClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                    )

                    is TransactionItemUIModel.AssetHead -> AssetListHead(
                        icon = row.icon,
                        onClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                    )

                    is TransactionItemUIModel.SwapHead -> SwapListHead(
                        fromAsset = row.fromAsset,
                        fromIcon = row.fromIcon,
                        fromValueText = row.fromValueText,
                        toAsset = row.toAsset,
                        toIcon = row.toIcon,
                        toValueText = row.toValueText,
                        fromEquivalentText = row.fromEquivalentText,
                        toEquivalentText = row.toEquivalentText,
                        onSwapClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                        onAssetClick = { onAction(TransactionAction.OpenAsset(it)) },
                    )

                    is TransactionItemUIModel.SwapAgain -> MainActionButton(
                        title = stringResource(R.string.transaction_swap_again),
                        modifier = Modifier.padding(horizontal = padding16, vertical = paddingSmall),
                        onClick = {
                            onAction(
                                TransactionAction.OpenSwap(
                                    fromAssetId = row.fromAssetId,
                                    toAssetId = row.toAssetId,
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
