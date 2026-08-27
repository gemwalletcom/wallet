package com.gemwallet.android.data.coordinators.confirm

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.confirm.coordinators.BuildConfirmProperties
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.isMemoSupport
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.Wallet

class BuildConfirmPropertiesImpl(
    private val stakeRepository: StakeRepository,
    private val explorerService: GemExplorerService,
) : BuildConfirmProperties {

    override suspend fun invoke(
        request: ConfirmParams,
        wallet: Wallet,
        assetsInfo: List<AssetInfo>,
        addressName: AddressName?,
    ): List<ConfirmProperty> {
        val assetInfo = assetsInfo.getByAssetId(request.assetId) ?: return emptyList()
        val chain = assetInfo.asset.id.chain
        return mutableListOf<ConfirmProperty?>().apply {
            add(ConfirmProperty.Source(wallet.name, wallet.type, assetInfo.owner?.chain, wallet.imageUrl))
            val destination = ConfirmProperty.Destination.map(request, getValidator(request), addressName)
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
            add(ConfirmProperty.Network(assetInfo.chain.asset()))
            add(
                ConfirmProperty.Memo(request.memo().orEmpty()).takeIf {
                    (request is ConfirmParams.TransferParams.Native || request is ConfirmParams.TransferParams.Token)
                            && assetInfo.asset.isMemoSupport()
                }
            )
        }.filterNotNull()
    }

    private suspend fun getValidator(params: ConfirmParams): DelegationValidator? {
        val validatorId = when (params) {
            is ConfirmParams.Stake.DelegateParams -> params.validator.id
            is ConfirmParams.Stake.RedelegateParams -> params.destinationValidator.id
            is ConfirmParams.Stake.UndelegateParams -> params.delegation.base.validatorId
            is ConfirmParams.Stake.WithdrawParams -> params.delegation.base.validatorId
            is ConfirmParams.Stake.RewardsParams -> params.validators.singleOrNull()?.id
            is ConfirmParams.Activate,
            is ConfirmParams.Stake.Freeze,
            is ConfirmParams.Stake.Unfreeze,
            is ConfirmParams.SwapParams,
            is ConfirmParams.NftParams,
            is ConfirmParams.PerpetualParams,
            is ConfirmParams.TransferParams -> null
        }
        return stakeRepository.getStakeValidator(params.assetId, validatorId ?: return null)
    }

    private fun List<AssetInfo>.getByAssetId(assetId: AssetId): AssetInfo? {
        return firstOrNull { it.id().toIdentifier() == assetId.toIdentifier() }
    }
}
