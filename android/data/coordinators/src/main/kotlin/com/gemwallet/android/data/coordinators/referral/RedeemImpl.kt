package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.referral.coordinators.Redeem
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.referral.values.ReferralError
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.RedemptionResult
import com.wallet.core.primitives.RewardRedemptionOption
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class RedeemImpl(
    private val sessionRepository: SessionRepository,
    private val rewardsService: GemRewardsService,
    private val getAuthPayload: GetAuthPayload,
    private val enableAsset: EnableAsset,
) : Redeem {

    override suspend fun redeem(wallet: Wallet, rewards: Rewards, option: RewardRedemptionOption): RedemptionResult {
        val authPayload = getAuthPayload.getAuthPayload(wallet)
        if (rewards.points < option.points) {
            throw ReferralError.InsufficientPoints
        }
        val result = rewardsService.redeem(
            walletId = wallet.id.id,
            auth = authPayload.toJson(),
            redemptionId = option.id,
        ).decodeJson<RedemptionResult>()
        sessionRepository.session().firstOrNull()?.let { session ->
            val assetId = option.asset?.id ?: return@let
            session.wallet.getAccount(assetId.chain) ?: return@let
            enableAsset(session.wallet.id, assetId)
        }
        return result
    }
}
