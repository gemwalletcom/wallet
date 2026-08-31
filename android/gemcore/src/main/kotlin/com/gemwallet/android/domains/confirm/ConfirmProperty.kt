package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.WalletType

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
                params: ConfirmParams,
                validator: DelegationValidator?,
                addressName: AddressName? = null,
            ): Destination? = when (params) {
                is ConfirmParams.Activate,
                is ConfirmParams.Stake.Freeze,
                is ConfirmParams.Stake.Unfreeze,
                is ConfirmParams.PerpetualParams,
                is ConfirmParams.SwapParams -> null
                is ConfirmParams.Stake.RewardsParams -> validator
                    ?.takeIf { params.validators.size == 1 }
                    ?.let { Stake(data = it.name, address = it.id) }
                is ConfirmParams.Stake.DelegateParams,
                is ConfirmParams.Stake.RedelegateParams,
                is ConfirmParams.Stake.UndelegateParams,
                is ConfirmParams.Stake.WithdrawParams -> Stake(data = validator?.name ?: "", address = validator?.id)
                is ConfirmParams.NftParams,
                is ConfirmParams.TransferParams.Deposit,
                is ConfirmParams.TransferParams.Withdrawal,
                is ConfirmParams.TransferParams.Transfer -> {
                    val destination = params.destination() ?: throw ConfirmError.RecipientEmpty
                    Transfer(
                        domain = destination.name ?: addressName?.name,
                        address = destination.address,
                        chain = params.assetId.chain,
                        addressType = addressName?.type,
                        imageUrl = addressName?.imageUrl,
                    )
                }
                is ConfirmParams.TransferParams.Generic -> Generic(params.metadata.name)
            }
        }
    }
}
