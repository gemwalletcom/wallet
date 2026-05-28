package com.gemwallet.android.features.activities.presents.details

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.features.activities.presents.details.components.DestinationPropertyItem
import com.gemwallet.android.features.activities.presents.details.components.SwapProgressItem
import com.gemwallet.android.features.activities.presents.details.components.TransactionExplorer
import com.gemwallet.android.features.activities.presents.details.components.TransactionRateProperty
import com.gemwallet.android.features.activities.presents.details.components.TransactionStatusProperty
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkFee
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.transaction.getTitle
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType

@Composable
internal fun TransactionDetailsScene(
    data: TransactionDetailsAggregate,
    onAction: (TransactionDetailsAction) -> Unit,
) {
    Scene(
        title = data.getTitle(),
        actions = {
            IconButton(onClick = { onAction(TransactionDetailsAction.Share) }) {
                Icon(AppIcons.Share, "")
            }
        },
        onClose = { onAction(TransactionDetailsAction.Close) },
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            data.valueGroups.forEach { group ->
                itemsPositioned(group.items) { position, item ->
                    when (item) {
                        is TransactionDetailsValue.Amount.NFT -> NftHead(
                            metadata = item.metadata,
                            onClick = {
                                onAction(TransactionDetailsAction.OpenNft(item.metadata.assetId))
                            },
                        )
                        TransactionDetailsValue.Amount.None -> {}
                        is TransactionDetailsValue.Amount.Plain -> AmountListHead(
                            icon = item.asset,
                            amount = item.value,
                            equivalent = item.equivalent,
                            onClick = data.amountAction(item.asset)?.let { action -> { onAction(action) } },
                        )
                        is TransactionDetailsValue.Amount.Swap -> SwapListHead(
                            fromAsset = item.fromAsset,
                            fromValue = item.fromValue,
                            toAsset = item.toAsset,
                            toValue = item.toValue,
                            currency = item.currency,
                            onSwapClick = {
                                onAction(
                                    TransactionDetailsAction.OpenSwap(
                                        fromAssetId = item.fromAsset.id(),
                                        toAssetId = item.toAsset.id(),
                                    )
                                )
                            },
                            onAssetClick = { onAction(TransactionDetailsAction.OpenAsset(it)) },
                        )
                        is TransactionDetailsValue.Date -> PropertyItem(R.string.transaction_date, item.data, listPosition = position)
                        is TransactionDetailsValue.Destination -> DestinationPropertyItem(item, position)
                        is TransactionDetailsValue.Explorer -> TransactionExplorer(
                            item.name,
                            item.url
                        )
                        is TransactionDetailsValue.Fee -> PropertyNetworkFee(
                            networkTitle = item.asset.name,
                            networkSymbol = item.asset.symbol,
                            feeCrypto = item.value,
                            feeFiat = item.equivalent,
                            variantsAvailable = true,
                            onClick = { onAction(TransactionDetailsAction.ShowFeeDetails) },
                        )
                        is TransactionDetailsValue.Memo -> PropertyItem(R.string.transfer_memo, item.data, listPosition = position)
                        is TransactionDetailsValue.ResourceType -> PropertyItem(
                            R.string.stake_resource,
                            item.data.resourceTitle(),
                            listPosition = position,
                        )
                        is TransactionDetailsValue.Network -> PropertyNetworkItem(item.data.chain, listPosition = position)
                        is TransactionDetailsValue.Pnl -> PropertyItem(stringResource(R.string.perpetual_pnl), item.value, dataColor = item.direction.color(), listPosition = position)
                        is TransactionDetailsValue.Price -> PropertyItem(R.string.asset_price, item.data, listPosition = position)
                        is TransactionDetailsValue.Status -> TransactionStatusProperty(data.asset, item, position)
                        is TransactionDetailsValue.Rate -> TransactionRateProperty(item, position)
                        is TransactionDetailsValue.SwapProgress -> SwapProgressItem(item)
                        is TransactionDetailsValue.SwapAgain -> MainActionButton(
                            title = stringResource(R.string.transaction_swap_again),
                            modifier = Modifier.padding(horizontal = padding16, vertical = paddingSmall),
                            onClick = {
                                onAction(
                                    TransactionDetailsAction.OpenSwap(
                                        fromAssetId = item.fromAssetId,
                                        toAssetId = item.toAssetId,
                                    )
                                )
                            },
                        )
                    }
                }
            }
        }
    }
}

private fun TransactionDetailsAggregate.amountAction(asset: Asset): TransactionDetailsAction.Navigation? {
    return when (type) {
        TransactionType.Transfer,
        TransactionType.TokenApproval,
        TransactionType.StakeDelegate,
        TransactionType.StakeUndelegate,
        TransactionType.StakeRewards,
        TransactionType.StakeRedelegate,
        TransactionType.StakeWithdraw,
        TransactionType.StakeFreeze,
        TransactionType.StakeUnfreeze -> TransactionDetailsAction.OpenAsset(asset.id)
        TransactionType.PerpetualOpenPosition,
        TransactionType.PerpetualClosePosition,
        TransactionType.PerpetualModifyPosition -> TransactionDetailsAction.OpenPerpetual(asset.id)
        TransactionType.Swap,
        TransactionType.TransferNFT,
        TransactionType.AssetActivation,
        TransactionType.SmartContractCall,
        TransactionType.EarnDeposit,
        TransactionType.EarnWithdraw -> null
    }
}

@Composable
private fun Resource.resourceTitle(): String = when (this) {
    Resource.Bandwidth -> stringResource(R.string.stake_resource_bandwidth)
    Resource.Energy -> stringResource(R.string.stake_resource_energy)
}
