package com.gemwallet.android.domains.transaction.values

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.TransactionState

sealed interface TransactionDetailsValue {

    sealed interface Amount : TransactionDetailsValue {
        class Swap(
            val fromAsset: AssetInfo,
            val fromValue: String,
            val toAsset: AssetInfo,
            val toValue: String,
            val currency: Currency,
        ) : Amount

        class NFT(val metadata: TransactionNFTTransferMetadata) : Amount

        class Plain(
            val asset: Asset,
            val value: String,
            val equivalent: String?,
        ) : Amount

        object None : Amount
    }

    class Fee(
        val asset: Asset,
        val value: String,
        val equivalent: String,
    ) : TransactionDetailsValue

    class Date(val data: String) : TransactionDetailsValue

    sealed class Destination(val data: String, open val explorerLink: BlockExplorerLink? = null) : TransactionDetailsValue {
        class Sender(data: String, override val explorerLink: BlockExplorerLink? = null) : Destination(data, explorerLink)
        class Recipient(data: String, override val explorerLink: BlockExplorerLink? = null) : Destination(data, explorerLink)
        class Contract(data: String, override val explorerLink: BlockExplorerLink? = null) : Destination(data, explorerLink)
        class Validator(data: String, override val explorerLink: BlockExplorerLink? = null) : Destination(data, explorerLink)
        class ProviderAddress(data: String, override val explorerLink: BlockExplorerLink? = null) : Destination(data, explorerLink)
        class Provider(name: String) : Destination(name)
    }

    class Status(val data: TransactionState) : TransactionDetailsValue

    class Memo(val data: String) : TransactionDetailsValue

    class Network(val data: Asset) : TransactionDetailsValue

    class Explorer(val url: String, val name: String) : TransactionDetailsValue
}
