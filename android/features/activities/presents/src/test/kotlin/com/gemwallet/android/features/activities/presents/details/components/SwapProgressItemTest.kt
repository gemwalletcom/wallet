package com.gemwallet.android.features.activities.presents.details.components

import com.gemwallet.android.ui.R
import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class SwapProgressItemTest {
    @Test
    fun revertedSwapShowsSourceRevertedAndSwapWaiting() {
        val statuses = TransactionState.Reverted.swapProgressStatuses()

        assertEquals(SwapProgressStatus.Reverted, statuses.transfer)
        assertEquals(R.string.transaction_status_reverted, statuses.transfer.labelRes())
        assertEquals(SwapProgressStatus.Waiting, statuses.swap)
        assertNull(statuses.swap.labelRes())
    }
}
