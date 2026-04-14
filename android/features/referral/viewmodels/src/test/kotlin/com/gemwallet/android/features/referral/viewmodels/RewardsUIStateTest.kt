package com.gemwallet.android.features.referral.viewmodels

import com.wallet.core.primitives.RewardStatus
import com.wallet.core.primitives.Rewards
import org.junit.Assert.assertEquals
import org.junit.Test

class RewardsUIStateTest {

    @Test
    fun testNullRewards() {
        val state = RewardsUIState.from(null)
        assertEquals(false, state.canInvite)
        assertEquals(false, state.isUnverified)
        assertEquals(false, state.hasPendingReferral)
        assertEquals(false, state.canActivatePendingReferral)
    }

    @Test
    fun testVerifiedCanInvite() {
        val state = RewardsUIState.from(rewards(status = RewardStatus.Verified, code = "user1"))
        assertEquals(true, state.canInvite)
        assertEquals(false, state.isUnverified)
    }

    @Test
    fun testTrustedCanInvite() {
        val state = RewardsUIState.from(rewards(status = RewardStatus.Trusted, code = "user1"))
        assertEquals(true, state.canInvite)
    }

    @Test
    fun testUnverifiedWithCode() {
        val state = RewardsUIState.from(rewards(status = RewardStatus.Unverified, code = "user1"))
        assertEquals(false, state.canInvite)
        assertEquals(true, state.isUnverified)
        assertEquals(false, state.hasPendingReferral)
    }

    @Test
    fun testUnverifiedWithoutCode() {
        val state = RewardsUIState.from(rewards(status = RewardStatus.Unverified, code = null))
        assertEquals(false, state.isUnverified)
    }

    @Test
    fun testPendingWithVerifyAfter() {
        val future = System.currentTimeMillis() + 86400000
        val state = RewardsUIState.from(rewards(status = RewardStatus.Pending, verifyAfter = future))
        assertEquals(false, state.canInvite)
        assertEquals(false, state.isUnverified)
        assertEquals(true, state.hasPendingReferral)
        assertEquals(false, state.canActivatePendingReferral)
    }

    @Test
    fun testPendingReadyToActivate() {
        val past = System.currentTimeMillis() - 1000
        val state = RewardsUIState.from(rewards(status = RewardStatus.Pending, verifyAfter = past, usedReferralCode = "ref1"))
        assertEquals(true, state.hasPendingReferral)
        assertEquals(true, state.canActivatePendingReferral)
    }

    @Test
    fun testDisabled() {
        val state = RewardsUIState.from(rewards(status = RewardStatus.Disabled, code = "user1"))
        assertEquals(false, state.canInvite)
        assertEquals(false, state.isUnverified)
    }

    private fun rewards(
        status: RewardStatus,
        code: String? = null,
        verifyAfter: Long? = null,
        usedReferralCode: String? = null,
    ) = Rewards(
        code = code,
        referralCount = 0,
        points = 0,
        usedReferralCode = usedReferralCode,
        status = status,
        verifyAfter = verifyAfter,
        redemptionOptions = emptyList(),
    )
}
