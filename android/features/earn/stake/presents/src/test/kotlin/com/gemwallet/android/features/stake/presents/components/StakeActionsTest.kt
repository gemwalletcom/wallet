package com.gemwallet.android.features.stake.presents.components

import com.gemwallet.android.features.stake.models.StakeAction
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class StakeActionsTest {
    @Test
    fun stakeAction_requiresValidators() {
        assertTrue(StakeAction.Stake.requiresValidators())
    }

    @Test
    fun nonStakeActions_doNotRequireValidators() {
        assertFalse(StakeAction.Freeze.requiresValidators())
        assertFalse(StakeAction.Unfreeze.requiresValidators())
        assertFalse(StakeAction.Rewards("1 SUI").requiresValidators())
    }
}
