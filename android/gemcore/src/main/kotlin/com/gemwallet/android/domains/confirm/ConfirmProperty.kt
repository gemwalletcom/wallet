package com.gemwallet.android.domains.confirm

import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeType
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData

sealed interface ConfirmProperty {
    class Source(val data: String, val walletType: WalletType, val walletChain: Chain?, val walletImageUrl: String?) : ConfirmProperty
    class Network(val data: Asset) : ConfirmProperty
    class Memo(memo: String) : ConfirmProperty {
        val data: String = memo.ifEmpty { "-" }
    }

    sealed class Destination(val data: String) : ConfirmProperty {
        class Stake(data: String, val address: String? = null, val explorerLink: BlockExplorerLink? = null) : Destination(data)
        class Provider(data: String) : Destination(data)
        class Transfer(
            val domain: String?,
            val address: String,
            val chain: Chain,
            val addressType: AddressType? = null,
            val imageUrl: String? = null,
            val explorerLink: BlockExplorerLink? = null,
        ) : Destination(address)
        class Generic(val appName: String) : Destination(appName)
        class PerpetualOper(val providerName: String) : Destination(providerName)

        companion object {
            fun map(
                transfer: GemTransferData,
                validator: DelegationValidator?,
                addressName: AddressName? = null,
            ): Destination? = when (val inputType = transfer.inputType) {
                is GemTransactionInputType.Account,
                is GemTransactionInputType.Perpetual,
                is GemTransactionInputType.TokenApprove,
                is GemTransactionInputType.Earn,
                is GemTransactionInputType.Swap -> null
                is GemTransactionInputType.Stake -> when (val stakeType = inputType.stakeType.decodeJson<StakeType>()) {
                    is StakeType.Freeze,
                    is StakeType.Unfreeze -> null
                    is StakeType.Rewards -> validator
                        ?.takeIf { stakeType.content.size == 1 }
                        ?.let { Stake(data = it.name, address = it.id) }
                    is StakeType.Stake,
                    is StakeType.Redelegate,
                    is StakeType.Unstake,
                    is StakeType.Withdraw -> Stake(data = validator?.name ?: "", address = validator?.id)
                }
                is GemTransactionInputType.TransferNft,
                is GemTransactionInputType.Deposit,
                is GemTransactionInputType.Withdrawal,
                is GemTransactionInputType.Transfer -> Transfer(
                    domain = transfer.recipient.name ?: addressName?.name,
                    address = transfer.recipient.address,
                    chain = inputType.chain,
                    addressType = addressName?.type,
                    imageUrl = addressName?.imageUrl,
                )
                is GemTransactionInputType.Generic -> Generic(inputType.metadata.decodeJson<ApplicationMetadata>().name)
            }
        }
    }
}
