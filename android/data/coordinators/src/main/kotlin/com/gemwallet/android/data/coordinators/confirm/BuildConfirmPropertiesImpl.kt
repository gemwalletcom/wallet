package com.gemwallet.android.data.coordinators.confirm

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.stakeType
import com.gemwallet.android.ext.asset
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeType
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemTransferData
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class BuildConfirmPropertiesImpl(
    private val getStakeValidator: GetStakeValidator,
    private val explorerService: GemExplorerService,
) : BuildConfirmProperties {

    override suspend fun invoke(
        transfer: GemTransferData,
        wallet: Wallet,
        addressName: AddressName?,
    ): List<ConfirmProperty> {
        val asset = transfer.inputType.asset
        val chain = asset.id.chain
        val validator = getValidator(transfer)
        return withContext(Dispatchers.IO) {
        mutableListOf<ConfirmProperty?>().apply {
            add(ConfirmProperty.Source(wallet.name, wallet.type, chain, wallet.imageUrl))
            val destination = ConfirmProperty.Destination.map(transfer, validator, addressName)
            add(
                when (destination) {
                    is ConfirmProperty.Destination.Transfer -> ConfirmProperty.Destination.Transfer(
                        domain = destination.domain,
                        address = destination.address,
                        chain = destination.chain,
                        addressType = destination.addressType,
                        imageUrl = destination.imageUrl,
                        explorerLink = explorerService.getAddressUrl(chain.string, destination.address).let { BlockExplorerLink(it.name, it.link) },
                    )
                    is ConfirmProperty.Destination.Stake -> destination.address?.let { address ->
                        ConfirmProperty.Destination.Stake(
                            data = destination.data,
                            address = address,
                            explorerLink = explorerService.getAddressUrl(chain.string, address).let { BlockExplorerLink(it.name, it.link) },
                        )
                    } ?: destination
                    else -> destination
                }
            )
            add(ConfirmProperty.Network(chain.asset()))
            add(ConfirmProperty.Memo(transfer.recipient.memo.orEmpty()).takeIf { transfer.inputType.showsMemo() })
        }.filterNotNull()
        }
    }

    private suspend fun getValidator(transfer: GemTransferData): DelegationValidator? {
        val inputType = transfer.inputType
        val validatorId = when (val stakeType = inputType.stakeType) {
            is StakeType.Stake -> stakeType.content.id
            is StakeType.Redelegate -> stakeType.content.toValidator.id
            is StakeType.Unstake -> stakeType.content.base.validatorId
            is StakeType.Withdraw -> stakeType.content.base.validatorId
            is StakeType.Rewards -> stakeType.content.singleOrNull()?.id
            is StakeType.Freeze,
            is StakeType.Unfreeze,
            null -> null
        }
        return getStakeValidator(inputType.asset.id, validatorId ?: return null)
    }
}
