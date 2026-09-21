package com.gemwallet.android.features.activities.presents.details

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
import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.features.activities.presents.details.components.SwapProgressItem
import com.gemwallet.android.features.activities.viewmodels.models.TransactionDetailsRowUIModel
import com.gemwallet.android.features.activities.viewmodels.models.TransactionHeaderTarget
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.AssetRatePropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.ChainAddress

@Composable
internal fun TransactionDetailsScene(data: TransactionDetailsAggregate, sections: List<ListSection<TransactionDetailsRowUIModel>>, headerTarget: TransactionHeaderTarget?, onAction: (TransactionDetailsAction) -> Unit) {
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current
    Scene(
        title = data.title.string(LocalContext.current),
        actions = {
            IconButton(onClick = { onAction(TransactionDetailsAction.Share) }) {
                Icon(AppIcons.Share, "")
            }
        },
        onClose = { onAction(TransactionDetailsAction.Close) },
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            listSections(sections) { position, row ->
                when (row) {
                    is TransactionDetailsRowUIModel.Item -> ListItem(
                        model = row.model,
                        listPosition = position,
                        modifier = row.url?.let { url -> Modifier.clickable { uriHandler.open(context, url) } } ?: Modifier,
                        accessory = row.url?.let { { DataBadgeChevron() } },
                    )

                    is TransactionDetailsRowUIModel.Address -> AddressPropertyItem(
                        title = row.title,
                        displayText = row.text,
                        copyValue = row.address,
                        explorerLink = row.explorerLink,
                        listPosition = position,
                        onClick = row.chain?.let { chain -> { onAction(TransactionDetailsAction.OpenAddress(ChainAddress(chain, row.address))) } },
                    )

                    is TransactionDetailsRowUIModel.Fee -> ListItem(
                        model = row.model,
                        listPosition = position,
                        modifier = Modifier.clickable { onAction(TransactionDetailsAction.ShowFeeDetails) },
                        accessory = { DataBadgeChevron() },
                    )

                    is TransactionDetailsRowUIModel.SwapProgress -> SwapProgressItem(row.model)

                    is TransactionDetailsRowUIModel.Row -> GemListRowView(row = row.row, listPosition = position, infoIcon = row.infoIcon)

                    is TransactionDetailsRowUIModel.Value -> when (val item = row.value) {
                        is TransactionDetailsValue.Amount.NFT -> NftHead(
                            metadata = item.metadata,
                            onClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                        )

                        is TransactionDetailsValue.Amount.Plain -> AmountListHead(
                            icon = item.asset,
                            amount = item.value,
                            equivalent = item.equivalent,
                            onClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                        )

                        is TransactionDetailsValue.Amount.Swap -> SwapListHead(
                            fromAsset = item.fromAsset,
                            fromValueText = item.fromValueText,
                            toAsset = item.toAsset,
                            toValueText = item.toValueText,
                            fromEquivalentText = item.fromEquivalentText,
                            toEquivalentText = item.toEquivalentText,
                            onSwapClick = headerTarget?.let { target -> { onAction(target.navigation()) } },
                            onAssetClick = { onAction(TransactionDetailsAction.OpenAsset(it)) },
                        )

                        is TransactionDetailsValue.Destination,
                        is TransactionDetailsValue.SwapProgress,
                        is TransactionDetailsValue.Fee,
                        is TransactionDetailsValue.Row,
                        is TransactionDetailsValue.EstimatedConfirmation,
                        -> Unit

                        is TransactionDetailsValue.Rate -> AssetRatePropertyItem(item.rate, position)

                        is TransactionDetailsValue.SwapAgain -> MainActionButton(
                            title = stringResource(R.string.transaction_swap_again),
                            modifier = Modifier.padding(horizontal = padding16, vertical = paddingSmall),
                            onClick = {
                                onAction(
                                    TransactionDetailsAction.OpenSwap(
                                        fromAssetId = item.fromAssetId,
                                        toAssetId = item.toAssetId,
                                    ),
                                )
                            },
                        )
                    }
                }
            }
        }
    }
}

private fun TransactionHeaderTarget.navigation(): TransactionDetailsAction.Navigation = when (this) {
    is TransactionHeaderTarget.Asset -> TransactionDetailsAction.OpenAsset(assetId)
    is TransactionHeaderTarget.Nft -> TransactionDetailsAction.OpenNft(assetId)
    is TransactionHeaderTarget.Swap -> TransactionDetailsAction.OpenSwap(fromAssetId = fromAssetId, toAssetId = toAssetId)
    is TransactionHeaderTarget.Perpetual -> TransactionDetailsAction.OpenPerpetual(assetId)
}
