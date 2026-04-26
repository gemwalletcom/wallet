package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.data.repositories.transactions.TransactionRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.domains.asset.getImageUrl
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ext.getNftMetadata
import com.gemwallet.android.ext.getSwapMetadata
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.model.format
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map

class GetTransactionsImpl(
    private val transactionsRepository: TransactionRepository,
) : GetTransactions {

    override fun getTransactions(
        assetId: AssetId?,
        state: TransactionState?,
        filterByChains: List<Chain>,
        filterByType: List<TransactionType>
    ): Flow<List<TransactionDataAggregate>> = transactionsRepository.getTransactions()
        .map { transactions ->
            transactions.filter { tx ->
                val byChain = filterByChains.isEmpty() ||
                    filterByChains.containsAll((tx.assets + listOf(tx.asset, tx.feeAsset)).map { it.chain }.toSet())
                val byType = filterByType.isEmpty() || filterByType.contains(tx.transaction.type)
                val byState = state == null || tx.transaction.state == state
                val byAssetId = assetId == null || tx.asset.id == assetId ||
                    tx.transaction.getSwapMetadata()?.toAsset == assetId ||
                    tx.transaction.getSwapMetadata()?.fromAsset == assetId
                byChain && byType && byState && byAssetId
            }
        }
        .map { items -> items.map { TransactionDataAggregateImpl(it) } }
        .flowOn(Dispatchers.IO)
}

@Stable
class TransactionDataAggregateImpl(
    private val data: TransactionExtended,
) : TransactionDataAggregate {

    override val id: String = data.transaction.id.identifier

    override val asset: Asset = data.asset

    private val participantAddressName: AddressName? = when (data.transaction.type) {
        TransactionType.StakeDelegate,
        TransactionType.StakeUndelegate,
        TransactionType.StakeRedelegate,
        TransactionType.EarnDeposit,
        TransactionType.EarnWithdraw -> data.toAddress
        else -> when (data.transaction.direction) {
            TransactionDirection.Incoming -> data.fromAddress
            TransactionDirection.Outgoing,
            TransactionDirection.SelfTransfer -> data.toAddress
        }
    }

    override val addressName: String? = participantAddressName?.name

    override val addressType: AddressType? = participantAddressName?.type

    override val address: String get() = when (data.transaction.type) {
        TransactionType.TransferNFT,
        TransactionType.Transfer,
        TransactionType.TokenApproval,
        TransactionType.SmartContractCall -> when (data.transaction.direction) {
            TransactionDirection.SelfTransfer,
            TransactionDirection.Outgoing -> AddressFormatter(data.transaction.to, chain = data.transaction.assetId.chain).value()
            TransactionDirection.Incoming -> AddressFormatter(data.transaction.from, chain = data.transaction.assetId.chain).value()
        }
        TransactionType.StakeDelegate,
        TransactionType.StakeUndelegate,
        TransactionType.StakeRedelegate,
        TransactionType.EarnDeposit,
        TransactionType.EarnWithdraw -> AddressFormatter(data.transaction.to, chain = data.transaction.assetId.chain).value()
        TransactionType.Swap,
        TransactionType.StakeWithdraw,
        TransactionType.AssetActivation,
        TransactionType.StakeRewards,
        TransactionType.PerpetualOpenPosition,
        TransactionType.StakeFreeze,
        TransactionType.StakeUnfreeze,
        TransactionType.PerpetualClosePosition,
        TransactionType.PerpetualModifyPosition
            -> ""
    }

    override val value: String get() = when (data.transaction.type) {
        TransactionType.Swap -> {
            getSwapMetadata(true)?.let { (metadata, asset) ->
                "+${asset.format(Crypto(metadata.toValue), decimalPlace = 2, dynamicPlace = true)}"
            } ?: ""
        }
        TransactionType.StakeUndelegate,
        TransactionType.StakeRewards,
        TransactionType.StakeRedelegate,
        TransactionType.StakeWithdraw,
        TransactionType.EarnWithdraw,
        TransactionType.StakeDelegate,
        TransactionType.EarnDeposit,
        TransactionType.PerpetualOpenPosition,
        TransactionType.StakeFreeze,
        TransactionType.StakeUnfreeze,
        TransactionType.PerpetualClosePosition -> getFormattedValue()
        TransactionType.Transfer -> {
            when (data.transaction.direction) {
                TransactionDirection.SelfTransfer,
                TransactionDirection.Outgoing -> "-${getFormattedValue()}"
                TransactionDirection.Incoming -> "+${getFormattedValue()}"
            }
        }
        TransactionType.TokenApproval -> data.asset.symbol
        TransactionType.TransferNFT,
        TransactionType.AssetActivation,
        TransactionType.SmartContractCall,
        TransactionType.PerpetualModifyPosition
            -> ""
    }

    override val equivalentValue: String? get() = when (data.transaction.type) {
        TransactionType.Swap -> getSwapMetadata(false)?.let { (metadata, asset) ->
            "-${
                asset.format(
                    Crypto(metadata.fromValue),
                    decimalPlace = 2,
                    dynamicPlace = true
                )
            }"
        }
        else -> null
    }

    override val nftImageUrl: String? = data.transaction.getNftMetadata()?.getImageUrl()

    override val type: TransactionType = data.transaction.type

    override val direction: TransactionDirection  = data.transaction.direction

    override val state: TransactionState = data.transaction.state
    override val createdAt: Long = data.transaction.createdAt

    private fun getSwapMetadata(toAsset: Boolean): Pair<TransactionSwapMetadata, Asset>? {
        val swapMetadata = data.transaction.getSwapMetadata() ?: return null
        val asset = if (toAsset) {
            data.assets.firstOrNull { swapMetadata.toAsset == it.id }
        } else {
            data.assets.firstOrNull { swapMetadata.fromAsset == it.id }
        } ?: return null

        return Pair(swapMetadata, asset)
    }

    private fun getFormattedValue(): String = data.asset.format(
        crypto = data.transaction.value.toBigInteger(),
        decimalPlace = 2,
        dynamicPlace = true,
    )
}
