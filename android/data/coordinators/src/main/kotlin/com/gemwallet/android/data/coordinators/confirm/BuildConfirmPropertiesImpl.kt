package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.coordinators.BuildConfirmProperties
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.isMemoSupport
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.Explorer

class BuildConfirmPropertiesImpl(
    private val stakeRepository: StakeRepository,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
) : BuildConfirmProperties {

    override suspend fun invoke(
        request: ConfirmParams,
        assetsInfo: List<AssetInfo>,
    ): List<ConfirmProperty> {
        val assetInfo = assetsInfo.getByAssetId(request.assetId) ?: return emptyList()
        val chain = assetInfo.asset.id.chain
        val explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(chain)
        val chainExplorer = Explorer(chain.string)
        return mutableListOf<ConfirmProperty?>().apply {
            add(ConfirmProperty.Source(assetInfo.walletName))
            val destination = ConfirmProperty.Destination.map(request, getValidator(request))
            add(
                if (destination is ConfirmProperty.Destination.Transfer) {
                    ConfirmProperty.Destination.Transfer(
                        domain = destination.domain,
                        address = destination.address,
                        explorerLink = BlockExplorerLink(explorerName, chainExplorer.getAddressUrl(explorerName, destination.address)),
                    )
                } else {
                    destination
                }
            )
            add(request.memo()?.takeIf {
                (request is ConfirmParams.TransferParams.Native || request is ConfirmParams.TransferParams.Token)
                        && assetInfo.asset.isMemoSupport()
                        && it.isNotEmpty()
            }?.let { ConfirmProperty.Memo(it) })
            add(ConfirmProperty.Network(assetInfo.chain.asset()))
        }.filterNotNull()
    }

    private suspend fun getValidator(params: ConfirmParams): DelegationValidator? {
        val validatorId = when (params) {
            is ConfirmParams.Stake.DelegateParams -> params.validator.id
            is ConfirmParams.Stake.RedelegateParams -> params.destinationValidator.id
            is ConfirmParams.Stake.UndelegateParams -> params.delegation.base.validatorId
            is ConfirmParams.Stake.WithdrawParams -> params.delegation.base.validatorId
            is ConfirmParams.Activate,
            is ConfirmParams.Stake.RewardsParams,
            is ConfirmParams.Stake.Freeze,
            is ConfirmParams.Stake.Unfreeze,
            is ConfirmParams.SwapParams,
            is ConfirmParams.TokenApprovalParams,
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
