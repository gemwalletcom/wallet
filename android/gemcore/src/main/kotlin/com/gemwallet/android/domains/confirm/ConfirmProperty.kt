package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.DelegationValidator

sealed interface ConfirmProperty {
    class Source(val data: String) : ConfirmProperty
    class Network(val data: Asset) : ConfirmProperty
    class Memo(val data: String) : ConfirmProperty

    sealed class Destination(val data: String) : ConfirmProperty {
        class Stake(data: String, val address: String? = null, val explorerLink: BlockExplorerLink? = null) : Destination(data)
        class Provider(data: String) : Destination(data)
        class Transfer(val domain: String?, val address: String, val explorerLink: BlockExplorerLink? = null) : Destination(address)
        class Generic(val appName: String) : Destination(appName)
        class PerpetualOper(val providerName: String) : Destination(providerName)

        companion object {
            fun map(params: ConfirmParams, validator: DelegationValidator?): Destination? = when (params) {
                is ConfirmParams.Activate,
                is ConfirmParams.Stake.Freeze,
                is ConfirmParams.Stake.Unfreeze,
                is ConfirmParams.PerpetualParams.Open,
                is ConfirmParams.PerpetualParams.Close,
                is ConfirmParams.PerpetualParams.Modify,
                is ConfirmParams.SwapParams -> null
                is ConfirmParams.Stake.RewardsParams -> validator
                    ?.takeIf { params.validators.size == 1 }
                    ?.let { Stake(data = it.name, address = it.id) }
                is ConfirmParams.Stake.DelegateParams,
                is ConfirmParams.Stake.RedelegateParams,
                is ConfirmParams.Stake.UndelegateParams,
                is ConfirmParams.Stake.WithdrawParams -> Stake(data = validator?.name ?: "", address = validator?.id)
                is ConfirmParams.TokenApprovalParams -> Provider(data = params.provider)
                is ConfirmParams.NftParams,
                is ConfirmParams.TransferParams.Token,
                is ConfirmParams.TransferParams.Native -> params.destination()?.let {
                    Transfer(domain = it.name, address = it.address)
                } ?: throw ConfirmError.RecipientEmpty
                is ConfirmParams.TransferParams.Generic -> Generic(params.name)
            }
        }
    }
}
