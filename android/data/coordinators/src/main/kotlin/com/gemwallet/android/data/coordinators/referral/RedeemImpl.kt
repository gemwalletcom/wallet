package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.referral.coordinators.Redeem
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.tokens.TokensRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.domains.referral.values.ReferralError
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.referralChain
import com.wallet.core.primitives.AuthenticatedRequest
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.RedemptionRequest
import com.wallet.core.primitives.RedemptionResult
import com.wallet.core.primitives.RewardRedemptionOption
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.firstOrNull

class RedeemImpl(
    private val sessionRepository: SessionRepository,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val getAuthPayload: GetAuthPayload,
    private val tokensRepository: TokensRepository,
    private val assetsRepository: AssetsRepository,
) : Redeem {

    override suspend fun redeem(wallet: Wallet, rewards: Rewards, option: RewardRedemptionOption): RedemptionResult {
        val account = wallet.getAccount(Chain.referralChain) ?: throw ReferralError.BadWallet
        val authPayload = getAuthPayload.getAuthPayload(wallet, account.chain)
        if (rewards.points < option.points) {
            throw ReferralError.InsufficientPoints
        }
        val result = gemDeviceApiClient.redeem(
            walletId = wallet.id,
            request = AuthenticatedRequest(
                auth = authPayload,
                data = RedemptionRequest(option.id)
            )
        )
        sessionRepository.session().firstOrNull()?.let {
            val assetId = option.asset?.id ?: return@let
            it.wallet.getAccount(assetId.chain) ?: return@let
            tokensRepository.search(assetId, it.currency)
            assetsRepository.switchVisibility(it.wallet.id, assetId, true)
        }
        return result
    }
}
