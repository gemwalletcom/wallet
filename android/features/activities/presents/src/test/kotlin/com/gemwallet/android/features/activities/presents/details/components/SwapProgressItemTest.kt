package com.gemwallet.android.features.activities.presents.details.components

import com.gemwallet.android.ui.R
import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Test

class SwapProgressItemTest {
    @Test
    fun revertedSwapShowsRefundedStatus() {
        val statuses = TransactionState.Reverted.swapProgressStatuses()

        assertEquals(SwapProgressStatus.Completed, statuses.transfer)
        assertEquals(SwapProgressStatus.Refunded, statuses.swap)
        assertEquals(R.string.transaction_status_refunded, statuses.swap.labelRes())
    }
}
