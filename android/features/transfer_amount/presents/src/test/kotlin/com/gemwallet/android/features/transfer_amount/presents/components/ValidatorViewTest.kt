package com.gemwallet.android.features.transfer_amount.presents.components

import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ValidatorViewTest {
    @Test
    fun stakeUndelegate_disablesValidatorSelection() {
        assertFalse(TransactionType.StakeUndelegate.canSelectValidatorOnAmountScreen())
    }

    @Test
    fun stakeDelegateAndRedelegate_enableValidatorSelection() {
        assertTrue(TransactionType.StakeDelegate.canSelectValidatorOnAmountScreen())
        assertTrue(TransactionType.StakeRedelegate.canSelectValidatorOnAmountScreen())
    }
}
