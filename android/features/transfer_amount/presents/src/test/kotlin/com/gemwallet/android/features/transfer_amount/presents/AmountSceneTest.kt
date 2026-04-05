package com.gemwallet.android.features.transfer_amount.presents

import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class AmountSceneTest {
    @Test
    fun transfer_keepsAmountInputTypeSwitch() {
        assertTrue(TransactionType.Transfer.canSwitchAmountInputTypeOnAmountScreen())
    }

    @Test
    fun stakeFlows_hideAmountInputTypeSwitch() {
        assertFalse(TransactionType.StakeDelegate.canSwitchAmountInputTypeOnAmountScreen())
        assertFalse(TransactionType.StakeUndelegate.canSwitchAmountInputTypeOnAmountScreen())
        assertFalse(TransactionType.StakeRedelegate.canSwitchAmountInputTypeOnAmountScreen())
    }
}
